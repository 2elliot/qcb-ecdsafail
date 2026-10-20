//! QCB audit instrument (V2): how much of a circuit's executed Toffoli count depends on the
//! classical offset bits?
//!
//! Usage: qcb_taint_count <path/to/ops.bin>
//!
//! 1. Static pass over the op stream (straight-line program, so one forward pass is exact):
//!    - classical bits carry a taint set {OFF = derived from offset_x/offset_y, MEAS = derived
//!      from an Hmr outcome}; taint flows through BitInvert/BitStore*/Hmr and the condition stack;
//!    - every CCX/CCZ gets a "condition class" = taint of its effective classical condition
//!      (op.c_condition AND every bit on the condition stack);
//!    - qubits are tracked as "classical-known" (value is a function of classical bits only)
//!      or quantum; every CCX/CCZ also gets a "control class" saying whether a control is a
//!      classical-known qubit carrying offset data (a Toffoli that could have been a
//!      classically conditioned CX on the leaderboard interface).
//! 2. Dynamic pass: replays the official scorer's Fiat-Shamir inputs and measurement RNG
//!    (copied from src/bin/eval_circuit.rs) with a re-implementation of src/sim.rs apply_iter
//!    that records executed Toffolis per op. Every batch is cross-checked against
//!    quantum_ecc::sim::Simulator: identical qubits, bits, phase and Toffoli total, or abort.
//!
//! Never modifies the harness. Prints a plain-text report on stdout.

use alloy_primitives::U256;
use quantum_ecc::circuit::{
    analyze_ops, BitId, Op, OperationType, QubitId, QubitOrBit, RegisterId, NO_BIT,
};
use quantum_ecc::sim::Simulator;
use quantum_ecc::weierstrass_elliptic_curve::WeierstrassEllipticCurve;
use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake256,
};
use std::fs::File;
use std::io::{BufReader, Read};

// ─── Copied from src/bin/eval_circuit.rs (trusted scorer) ──────────────────

const MAGIC: &[u8; 8] = b"QECCOPSZ";
const ZSTD_WINDOW_LOG_MAX: u32 = 27;
const OP_BYTES: usize = 56;
const MAX_OPS: u64 = 4_000_000_000;
const NUM_TESTS: usize = 9024;

fn op_kind_from_u32(v: u32) -> Option<OperationType> {
    Some(match v {
        0 => OperationType::Neg,
        1 => OperationType::Register,
        2 => OperationType::AppendToRegister,
        3 => OperationType::BitInvert,
        4 => OperationType::BitStore0,
        5 => OperationType::BitStore1,
        6 => OperationType::X,
        7 => OperationType::Z,
        8 => OperationType::CX,
        9 => OperationType::CZ,
        10 => OperationType::Swap,
        11 => OperationType::R,
        12 => OperationType::Hmr,
        13 => OperationType::CCX,
        14 => OperationType::CCZ,
        15 => OperationType::PushCondition,
        16 => OperationType::PopCondition,
        17 => OperationType::DebugPrint,
        _ => return None,
    })
}

fn read_u64(bytes: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(bytes[off..off + 8].try_into().unwrap())
}

fn load_ops(path: &str) -> Result<Vec<Op>, String> {
    let mut file = File::open(path).map_err(|e| format!("open {path}: {e}"))?;
    let mut header = [0u8; MAGIC.len() + 8];
    file.read_exact(&mut header)
        .map_err(|e| format!("{path}: too short to read header: {e}"))?;
    if &header[..MAGIC.len()] != MAGIC {
        return Err(format!("{path}: bad magic"));
    }
    let n = u64::from_le_bytes(header[MAGIC.len()..].try_into().unwrap());
    if n > MAX_OPS {
        return Err(format!("{path}: op count {n} exceeds cap {MAX_OPS}"));
    }
    let n = n as usize;
    let mut dec = zstd::stream::read::Decoder::new(BufReader::new(file))
        .map_err(|e| format!("{path}: zstd init: {e}"))?;
    dec.window_log_max(ZSTD_WINDOW_LOG_MAX)
        .map_err(|e| format!("{path}: zstd window cap: {e}"))?;
    let mut ops = Vec::with_capacity(n);
    let mut rec = [0u8; OP_BYTES];
    for i in 0..n {
        dec.read_exact(&mut rec)
            .map_err(|e| format!("op {i}: short read from compressed body: {e}"))?;
        let kind_raw = u32::from_le_bytes(rec[0..4].try_into().unwrap());
        let kind = op_kind_from_u32(kind_raw).ok_or_else(|| format!("op {i}: unknown kind {kind_raw}"))?;
        let op = Op {
            kind,
            q_control2: QubitId(read_u64(&rec, 8)),
            q_control1: QubitId(read_u64(&rec, 16)),
            q_target: QubitId(read_u64(&rec, 24)),
            c_target: BitId(read_u64(&rec, 32)),
            c_condition: BitId(read_u64(&rec, 40)),
            r_target: RegisterId(read_u64(&rec, 48)),
        };
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| op.validate())).is_err() {
            return Err(format!("op {i}: failed Op::validate()"));
        }
        ops.push(op);
    }
    let mut extra = [0u8; 1];
    match dec.read(&mut extra) {
        Ok(0) => {}
        Ok(_) => return Err(format!("{path}: trailing data after {n} ops")),
        Err(e) => return Err(format!("{path}: error checking for trailing data: {e}")),
    }
    Ok(ops)
}

fn secp256k1() -> WeierstrassEllipticCurve {
    let h = |s: &str| U256::from_str_radix(s, 16).unwrap();
    WeierstrassEllipticCurve {
        modulus: h("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F"),
        a: U256::from(0),
        b: U256::from(7),
        gx: h("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798"),
        gy: h("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8"),
        order: h("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141"),
    }
}

fn fiat_shamir_seed(ops: &[Op]) -> sha3::Shake256Reader {
    let mut hasher = Shake256::default();
    hasher.update(b"quantum_ecc-fiat-shamir-v2");
    hasher.update(&(ops.len() as u64).to_le_bytes());
    for op in ops {
        hasher.update(&[op.kind as u8]);
        hasher.update(&op.q_control2.0.to_le_bytes());
        hasher.update(&op.q_control1.0.to_le_bytes());
        hasher.update(&op.q_target.0.to_le_bytes());
        hasher.update(&op.c_target.0.to_le_bytes());
        hasher.update(&op.c_condition.0.to_le_bytes());
        hasher.update(&op.r_target.0.to_le_bytes());
    }
    hasher.finalize_xof()
}

type Pt = (U256, U256);

/// Same draw and filters as eval_circuit.rs run_tests (lines 245-270).
fn gen_inputs(xof: &mut sha3::Shake256Reader) -> (Vec<Pt>, Vec<Pt>, Vec<Pt>) {
    let curve = secp256k1();
    let (mut ts, mut os, mut es) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..NUM_TESTS {
        let mut rb = [[0u8; 32]; 2];
        XofReader::read(xof, &mut rb[0]);
        XofReader::read(xof, &mut rb[1]);
        let k1 = U256::from_le_bytes(rb[0]);
        let k2 = U256::from_le_bytes(rb[1]);
        let t = curve.mul(curve.gx, curve.gy, k1);
        let o = curve.mul(curve.gx, curve.gy, k2);
        if t.0 == o.0 || (t.0.is_zero() && t.1.is_zero()) || (o.0.is_zero() && o.1.is_zero()) {
            continue;
        }
        es.push(curve.add(t.0, t.1, o.0, o.1));
        ts.push(t);
        os.push(o);
    }
    (ts, os, es)
}

// ─── Static taint pass ─────────────────────────────────────────────────────

const OFF: u8 = 1; // depends on an offset input bit
const MEAS: u8 = 2; // depends on a measurement outcome

// Control-class bits for CCX/CCZ.
const CTRL_OFF: u8 = 1; // a control is a classical-known qubit carrying offset data
const CTRL_CONST: u8 = 2; // a control is a classical-known constant (no taint)
const CTRL_MEAS: u8 = 4; // a control is classical-known, measurement-derived only

struct Static {
    cond_class: Vec<u8>,   // per op: taint of the effective classical condition
    ctrl_class: Vec<u8>,   // per CCX/CCZ: CTRL_* bitmask
    off_x: Vec<bool>,      // per op: X on a qubit under an OFF-tainted condition (A load/unload)
    off_bitops: usize,     // Bit*/Push ops whose effect depends on OFF (free classical compute)
    off_bits_left: usize,  // non-register bits still OFF-tainted at end of stream
    off_by_kind: [usize; 18], // per OperationType: ops whose effective condition is OFF-tainted
    // For the windowed "restore" estimate (V4): which condition factors are OFF-tainted.
    push_off: Vec<bool>, // per PushCondition op: the pushed bit is OFF-tainted
    cc_off: Vec<bool>,   // per op: op.c_condition is OFF-tainted
    k_off: Vec<u8>,      // per op: number of OFF-tainted factors in its effective condition
}

fn classify(ops: &[Op], regs: &[Vec<QubitOrBit>], nq: usize, nb: usize) -> Static {
    let mut bt = vec![0u8; nb];
    let mut reg_bit = vec![false; nb];
    for r in &regs[2..4] {
        for x in r {
            if let QubitOrBit::Bit(b) = x {
                bt[b.0 as usize] = OFF;
                reg_bit[b.0 as usize] = true;
            }
        }
    }
    // Every qubit starts at |0> (clear_for_shot) except the target registers, which are quantum.
    let mut qc = vec![true; nq];
    let mut qt = vec![0u8; nq];
    for r in &regs[0..2] {
        for x in r {
            if let QubitOrBit::Qubit(q) = x {
                qc[q.0 as usize] = false;
            }
        }
    }

    let mut st = Static {
        cond_class: vec![0; ops.len()],
        ctrl_class: vec![0; ops.len()],
        off_x: vec![false; ops.len()],
        off_bitops: 0,
        off_bits_left: 0,
        off_by_kind: [0; 18],
        push_off: vec![false; ops.len()],
        cc_off: vec![false; ops.len()],
        k_off: vec![0; ops.len()],
    };
    // Stack entries: (taint before this push, pushed bit was OFF-tainted).
    let mut stack: Vec<(u8, bool)> = Vec::new();
    let mut stack_t = 0u8;
    let mut stack_off = 0u8; // number of OFF-tainted bits currently pushed
    let qi = |q: QubitId| q.0 as usize;

    for (i, op) in ops.iter().enumerate() {
        let conditioned = !stack.is_empty() || op.c_condition != NO_BIT;
        let cc_t = if op.c_condition != NO_BIT { bt[op.c_condition.0 as usize] } else { 0 };
        let ct = stack_t | cc_t;
        st.cond_class[i] = ct;
        if op.kind != OperationType::PushCondition {
            st.cc_off[i] = cc_t & OFF != 0;
            st.k_off[i] = stack_off + u8::from(cc_t & OFF != 0);
        }
        if ct & OFF != 0 {
            st.off_by_kind[op.kind as usize] += 1;
        }
        match op.kind {
            OperationType::PushCondition => {
                let t = bt[op.c_condition.0 as usize];
                let off = t & OFF != 0;
                st.push_off[i] = off;
                stack.push((stack_t, off));
                stack_t |= t;
                stack_off += u8::from(off);
                if off {
                    st.off_bitops += 1;
                }
            }
            OperationType::PopCondition => {
                if let Some((t, off)) = stack.pop() {
                    stack_t = t;
                    stack_off -= u8::from(off);
                }
            }
            OperationType::BitInvert => {
                let b = op.c_target.0 as usize;
                bt[b] |= ct;
                if bt[b] & OFF != 0 {
                    st.off_bitops += 1;
                }
            }
            OperationType::BitStore0 | OperationType::BitStore1 => {
                let b = op.c_target.0 as usize;
                bt[b] = if conditioned { bt[b] | ct } else { 0 };
                if bt[b] & OFF != 0 {
                    st.off_bitops += 1;
                }
            }
            OperationType::Hmr => {
                let b = op.c_target.0 as usize;
                bt[b] = MEAS | if conditioned { bt[b] | ct } else { 0 };
                let t = qi(op.q_target);
                if conditioned {
                    qt[t] |= ct;
                } else {
                    qc[t] = true;
                    qt[t] = 0;
                }
            }
            OperationType::R => {
                let t = qi(op.q_target);
                if conditioned {
                    qt[t] |= ct;
                } else {
                    qc[t] = true;
                    qt[t] = 0;
                }
            }
            OperationType::X => {
                let t = qi(op.q_target);
                if qc[t] {
                    qt[t] |= ct;
                }
                if ct & OFF != 0 {
                    st.off_x[i] = true;
                }
            }
            OperationType::CX => {
                let (c, t) = (qi(op.q_control1), qi(op.q_target));
                qc[t] = qc[t] && qc[c];
                qt[t] |= qt[c] | ct;
            }
            OperationType::Swap => {
                let (a, t) = (qi(op.q_control1), qi(op.q_target));
                if conditioned {
                    let c = qc[a] && qc[t];
                    let u = qt[a] | qt[t] | ct;
                    qc[a] = c;
                    qc[t] = c;
                    qt[a] = u;
                    qt[t] = u;
                } else {
                    qc.swap(a, t);
                    qt.swap(a, t);
                }
            }
            OperationType::CCX | OperationType::CCZ => {
                let mut cls = 0u8;
                for c in [op.q_control1, op.q_control2] {
                    let c = qi(c);
                    if qc[c] {
                        cls |= if qt[c] & OFF != 0 {
                            CTRL_OFF
                        } else if qt[c] == 0 {
                            CTRL_CONST
                        } else {
                            CTRL_MEAS
                        };
                    }
                }
                st.ctrl_class[i] = cls;
                if op.kind == OperationType::CCX {
                    let (c1, c2, t) = (qi(op.q_control1), qi(op.q_control2), qi(op.q_target));
                    qc[t] = qc[t] && qc[c1] && qc[c2];
                    qt[t] |= qt[c1] | qt[c2] | ct;
                }
            }
            OperationType::Z
            | OperationType::CZ
            | OperationType::Neg
            | OperationType::Register
            | OperationType::AppendToRegister
            | OperationType::DebugPrint => {}
        }
    }
    st.off_bits_left = (0..nb).filter(|&b| !reg_bit[b] && bt[b] & OFF != 0).count();
    st
}

// ─── Dynamic pass: src/sim.rs apply_iter with per-op Toffoli counters ─────

fn rng<R: XofReader>(xof: &mut R) -> u64 {
    let mut buf = [0u8; 8];
    XofReader::read(xof, &mut buf);
    u64::from_le_bytes(buf)
}

#[allow(clippy::too_many_arguments)]
fn apply_counted<R: XofReader>(
    ops: &[Op],
    st: &Static,
    q: &mut [u64],
    b: &mut [u64],
    phase: &mut u64,
    xof: &mut R,
    exec: &mut [u64],
    rest_exec: &mut [u64],
) {
    let mut stack: Vec<(u64, u64)> = Vec::new();
    let mut base = u64::MAX;
    // AND of the pushed bits that are NOT OFF-tainted: the part of the condition that would
    // stay classical if the addend were quantum.
    let mut base_rest = u64::MAX;
    for (i, op) in ops.iter().enumerate() {
        let mut cond = base;
        if op.c_condition != NO_BIT {
            cond &= b[op.c_condition.0 as usize];
        }
        if st.k_off[i] > 0 {
            let mut rest = base_rest;
            if op.c_condition != NO_BIT && !st.cc_off[i] {
                rest &= b[op.c_condition.0 as usize];
            }
            rest_exec[i] += rest.count_ones() as u64;
        }
        let t = op.q_target.0 as usize;
        let c1 = op.q_control1.0 as usize;
        let c2 = op.q_control2.0 as usize;
        match op.kind {
            OperationType::CCX => {
                exec[i] += cond.count_ones() as u64;
                let v = cond & q[c1] & q[c2];
                q[t] ^= v;
            }
            OperationType::CCZ => {
                exec[i] += cond.count_ones() as u64;
                *phase ^= cond & q[t] & q[c1] & q[c2];
            }
            OperationType::CX => {
                let v = cond & q[c1];
                q[t] ^= v;
            }
            OperationType::Swap => {
                let mut a = q[c1];
                let mut tt = q[t];
                a ^= tt;
                tt ^= cond & a;
                a ^= tt;
                q[c1] = a;
                q[t] = tt;
            }
            OperationType::X => q[t] ^= cond,
            OperationType::Z => *phase ^= cond & q[t],
            OperationType::CZ => *phase ^= cond & q[t] & q[c1],
            OperationType::Neg => *phase ^= cond,
            OperationType::Hmr => {
                let r = rng(xof);
                let ctb = op.c_target.0 as usize;
                b[ctb] &= !cond;
                b[ctb] ^= r & cond;
                *phase ^= q[t] & r & cond;
                q[t] &= !cond;
            }
            OperationType::R => {
                let r = rng(xof);
                *phase ^= q[t] & r & cond;
                q[t] &= !cond;
            }
            OperationType::BitInvert => b[op.c_target.0 as usize] ^= cond,
            OperationType::BitStore0 => b[op.c_target.0 as usize] &= !cond,
            OperationType::BitStore1 => b[op.c_target.0 as usize] |= cond,
            OperationType::PushCondition => {
                stack.push((base, base_rest));
                let v = b[op.c_condition.0 as usize];
                base &= v;
                if !st.push_off[i] {
                    base_rest &= v;
                }
            }
            OperationType::PopCondition => {
                if let Some((v, r)) = stack.pop() {
                    base = v;
                    base_rest = r;
                }
            }
            OperationType::AppendToRegister | OperationType::Register | OperationType::DebugPrint => {}
        }
    }
}

fn set_reg(q: &mut [u64], b: &mut [u64], reg: &[QubitOrBit], val: U256, shot: usize) {
    for (i, item) in reg.iter().enumerate() {
        let on = val.bit(i);
        let w = match item {
            QubitOrBit::Qubit(id) => &mut q[id.0 as usize],
            QubitOrBit::Bit(id) => &mut b[id.0 as usize],
        };
        if on {
            *w |= 1 << shot;
        } else {
            *w &= !(1 << shot);
        }
    }
}

fn get_reg(q: &[u64], reg: &[QubitOrBit], shot: usize) -> U256 {
    let mut v = U256::ZERO;
    for (i, item) in reg.iter().enumerate() {
        if let QubitOrBit::Qubit(id) = item {
            v.set_bit(i, (q[id.0 as usize] >> shot) & 1 != 0);
        }
    }
    v
}

// ─── Report ────────────────────────────────────────────────────────────────

fn main() {
    let path = std::env::args().nth(1).expect("usage: qcb_taint_count <ops.bin>");
    let ops = load_ops(&path).unwrap_or_else(|e| panic!("{e}"));
    let (nq, nb, _nr, regs) = analyze_ops(ops.iter());
    assert!(regs.len() == 4 && regs.iter().all(|r| r.len() == 256), "expected 4 x 256 registers");
    let (nq, nb) = (nq as usize, nb as usize);
    println!("ops.bin             : {path}");
    println!("ops                 : {}", ops.len());
    println!("qubits (Q)          : {nq}");
    println!("classical bits      : {nb}");

    // Static.
    let st = classify(&ops, &regs, nq, nb);
    let is_tof = |o: &Op| matches!(o.kind, OperationType::CCX | OperationType::CCZ);
    let n_ccx = ops.iter().filter(|o| o.kind == OperationType::CCX).count();
    let n_ccz = ops.iter().filter(|o| o.kind == OperationType::CCZ).count();
    println!("\n== static ==");
    println!("CCX / CCZ emitted   : {n_ccx} / {n_ccz}");
    let cname = ["unconditioned-or-const", "OFF", "MEAS", "OFF+MEAS"];
    for c in 0..4u8 {
        let n = ops.iter().enumerate().filter(|(i, o)| is_tof(o) && st.cond_class[*i] == c).count();
        println!("  CCX+CCZ cond class {:<24}: {n}", cname[c as usize]);
    }
    for (bit, name) in [(CTRL_OFF, "ctrl classical-OFF"), (CTRL_CONST, "ctrl classical-const"), (CTRL_MEAS, "ctrl classical-MEAS")] {
        let n = ops.iter().enumerate().filter(|(i, o)| is_tof(o) && st.ctrl_class[*i] & bit != 0).count();
        println!("  CCX+CCZ with {:<29}: {n}", name);
    }
    println!("offset-conditioned X ops (A load/unload) : {}", st.off_x.iter().filter(|&&x| x).count());
    println!("OFF-dependent classical bit/push ops     : {}", st.off_bitops);
    println!("non-register bits OFF-tainted at end     : {}", st.off_bits_left);
    println!("ops whose effective condition is OFF-tainted, by kind:");
    for k in 0..18u32 {
        if st.off_by_kind[k as usize] > 0 {
            println!("  {:<16?}: {}", op_kind_from_u32(k).unwrap(), st.off_by_kind[k as usize]);
        }
    }
    if std::env::args().any(|a| a == "--static") {
        return;
    }

    // Runs of offset-conditioned X (A loads/unloads): gap <= 4 ops joins a run.
    let mut runs: Vec<(usize, usize, usize)> = Vec::new();
    for (i, &x) in st.off_x.iter().enumerate() {
        if !x {
            continue;
        }
        match runs.last_mut() {
            Some(r) if i - r.1 <= 4 => {
                r.1 = i;
                r.2 += 1;
            }
            _ => runs.push((i, i, 1)),
        }
    }

    // Dynamic.
    let mut xof_a = fiat_shamir_seed(&ops);
    let (ts, os, es) = gen_inputs(&mut xof_a);
    let mut xof_b = fiat_shamir_seed(&ops);
    let mut skip = vec![0u8; 64 * NUM_TESTS];
    XofReader::read(&mut xof_b, &mut skip);
    let n = ts.len();

    let mut exec = vec![0u64; ops.len()];
    let mut rest_exec = vec![0u64; ops.len()];
    let mut q = vec![0u64; nq];
    let mut b = vec![0u64; nb];
    let mut official = Simulator::new(nq, nb, &mut xof_b);
    let mut fails = (0usize, 0usize, 0usize);
    let batches = (n + 63) / 64;
    for batch in 0..batches {
        let bs = 64.min(n - batch * 64);
        let mask: u64 = if bs == 64 { u64::MAX } else { (1u64 << bs) - 1 };
        q.iter_mut().for_each(|v| *v = 0);
        b.iter_mut().for_each(|v| *v = 0);
        let mut phase = 0u64;
        official.clear_for_shot();
        for s in 0..bs {
            let i = batch * 64 + s;
            for (r, v) in [(0, ts[i].0), (1, ts[i].1), (2, os[i].0), (3, os[i].1)] {
                set_reg(&mut q, &mut b, &regs[r], v, s);
                official.set_register(&regs[r], v, s);
            }
        }
        apply_counted(&ops, &st, &mut q, &mut b, &mut phase, &mut xof_a, &mut exec, &mut rest_exec);
        official.apply_iter(ops.iter());

        let my_tof: u64 = exec.iter().sum();
        assert_eq!(my_tof, official.stats.toffoli_gates, "Toffoli total diverged at batch {batch}");
        assert!(q == official.qubits && b == official.bits && phase == official.phase, "state diverged at batch {batch}");

        for s in 0..bs {
            let i = batch * 64 + s;
            if get_reg(&q, &regs[0], s) != es[i].0 || get_reg(&q, &regs[1], s) != es[i].1 {
                fails.0 += 1;
            }
        }
        if phase & mask != 0 {
            fails.1 += 1;
        }
        for r in &regs[0..2] {
            for x in r {
                if let QubitOrBit::Qubit(id) = x {
                    q[id.0 as usize] = 0;
                }
            }
        }
        if q.iter().any(|v| v & mask != 0) {
            fails.2 += 1;
        }
    }

    let total: u64 = exec.iter().sum();
    let avg = |s: u64| s as f64 / n.max(1) as f64;
    println!("\n== dynamic (official inputs, cross-checked vs quantum_ecc::sim::Simulator every batch) ==");
    println!("shots n             : {n}");
    println!("classical mismatches / phase-garbage batches / ancilla-garbage batches : {} / {} / {}", fails.0, fails.1, fails.2);
    println!("avg executed Toffoli T : {:.3}", avg(total));
    for c in 0..4u8 {
        let s: u64 = (0..ops.len()).filter(|&i| st.cond_class[i] == c).map(|i| exec[i]).sum();
        println!("  executed, cond class {:<24}: {:.3}", cname[c as usize], avg(s));
    }
    for (bit, name) in [(CTRL_OFF, "ctrl classical-OFF"), (CTRL_CONST, "ctrl classical-const"), (CTRL_MEAS, "ctrl classical-MEAS")] {
        let s: u64 = (0..ops.len()).filter(|&i| st.ctrl_class[i] & bit != 0).map(|i| exec[i]).sum();
        println!("  executed, {:<32}: {:.3}", name, avg(s));
    }

    println!("\n== A load/unload runs (offset-conditioned X) and the arithmetic between them ==");
    println!("{:>4} {:>11} {:>11} {:>6} | {:>14} {:>14}", "run", "start", "end", "X ops", "CCX+CCZ before", "avg exec before");
    let mut lo = 0usize;
    for (k, r) in runs.iter().enumerate() {
        let emitted = (lo..r.0).filter(|&i| is_tof(&ops[i])).count();
        let ex: u64 = exec[lo..r.0].iter().sum();
        println!("{:>4} {:>11} {:>11} {:>6} | {:>14} {:>14.1}", k, r.0, r.1, r.2, emitted, avg(ex));
        lo = r.1 + 1;
    }
    let emitted = (lo..ops.len()).filter(|&i| is_tof(&ops[i])).count();
    let ex: u64 = exec[lo..].iter().sum();
    println!("tail {:>11} {:>11} {:>6} | {:>14} {:>14.1}", lo, ops.len(), "", emitted, avg(ex));

    println!("\n== CCX/CCZ with a classical-known OFF control (first 50) ==");
    for i in (0..ops.len()).filter(|&i| st.ctrl_class[i] & CTRL_OFF != 0).take(50) {
        let o = &ops[i];
        let run = runs.iter().position(|r| r.0 > i).unwrap_or(runs.len());
        println!(
            "op {i:>9} {:?} c1=q{} c2=q{} t=q{} cond_class={} before_run={run} avg_exec={:.3}",
            o.kind, o.q_control1.0, o.q_control2.0, o.q_target.0, st.cond_class[i], avg(exec[i])
        );
    }

    // V4: what a quantum-addend port would pay back for gates the leaderboard lets the
    // circuit condition on OFF-derived classical bits. Executions are MEASURED: `exec` under
    // the full condition (leaderboard), `rest_exec` under the condition with every OFF-tainted
    // factor removed (those factors become quantum controls). The per-execution Toffoli
    // cost of k extra quantum controls is a MODEL (AND the k controls with k-1 Toffolis,
    // measurement-based uncompute, then one more control on the gate):
    //   X, Z: k-1 | CX, CZ, Swap: k | CCX, CCZ: k (plus the gate's own Toffoli on the
    //   executions the leaderboard skipped) | Neg: max(k-2, 0) | R, Hmr, Bit*, Push/Pop: 0.
    // k = 1 X gates are the addend loads; their windowed cost is the lookup, priced
    // separately (V3), so they contribute 0 here.
    println!("\n== windowed restore estimate (executions measured; per-execution cost modelled) ==");
    println!(
        "{:<14} {:>8} {:>6} {:>16} {:>16} {:>14}",
        "kind", "ops", "max k", "avg exec (LB)", "avg exec (win)", "extra T/shot"
    );
    let mut total_extra = 0f64;
    for k in 0..18u32 {
        let kind = op_kind_from_u32(k).unwrap();
        let idx: Vec<usize> = (0..ops.len()).filter(|&i| ops[i].kind == kind && st.k_off[i] > 0).collect();
        if idx.is_empty() {
            continue;
        }
        let maxk = idx.iter().map(|&i| st.k_off[i]).max().unwrap();
        let lb: u64 = idx.iter().map(|&i| exec_any(&ops[i], &exec, &rest_exec, i, false)).sum();
        let win: u64 = idx.iter().map(|&i| rest_exec[i]).sum();
        let extra: f64 = idx
            .iter()
            .map(|&i| {
                let k = st.k_off[i] as f64;
                let per = match kind {
                    OperationType::X | OperationType::Z => k - 1.0,
                    OperationType::CX | OperationType::CZ | OperationType::Swap => k,
                    OperationType::CCX | OperationType::CCZ => k,
                    OperationType::Neg => (k - 2.0).max(0.0),
                    _ => 0.0,
                };
                let base = if matches!(kind, OperationType::CCX | OperationType::CCZ) {
                    rest_exec[i] as f64 - exec[i] as f64
                } else {
                    0.0
                };
                per * rest_exec[i] as f64 + base
            })
            .sum();
        total_extra += extra;
        println!(
            "{:<14} {:>8} {:>6} {:>16.3} {:>16.3} {:>14.3}",
            format!("{kind:?}"), idx.len(), maxk, avg(lb), avg(win), avg_f(extra, n)
        );
    }
    println!("TOTAL windowed restore estimate: {:.3} extra Toffoli/shot", avg_f(total_extra, n));
}

/// Leaderboard executions for display: Toffoli ops have `exec`; others were not counted
/// by the harness, so show `rest_exec` filtered to 0 (they cost 0 T on the leaderboard).
fn exec_any(op: &Op, exec: &[u64], _rest: &[u64], i: usize, _w: bool) -> u64 {
    if matches!(op.kind, OperationType::CCX | OperationType::CCZ) {
        exec[i]
    } else {
        0
    }
}

fn avg_f(x: f64, n: usize) -> f64 {
    x / n.max(1) as f64
}
