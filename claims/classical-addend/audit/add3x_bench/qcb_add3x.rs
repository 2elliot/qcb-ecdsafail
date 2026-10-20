//! QCB audit P2: what does the incumbent's free classical 3·x_A precomputation save?
//!
//! Builds two circuits with the incumbent's (60d61859) own public test builders in
//! `trailmix_ludicrous/ec_add.rs`:
//!   - `build_add3x_test` (326): the shipped `coord_add3x` — 3·ox mod p computed with
//!     classical bit ops, loaded with conditioned X, one `mod_add`;
//!   - `build_add3x_test_orig` (353): `coord_add3x_orig` — ox loaded once, then
//!     `mod_add, mod_double, mod_add, mod_double_reverse` (no classical 3·ox, so it
//!     also works for a quantum addend).
//! Each is simulated with the harness `Simulator` on random (dst, ox) with dst, ox < p.
//! Checks dst' == dst + 3·ox mod p, every non-dst qubit back at |0>, zero phase; reports
//! executed Toffolis per shot.
//!
//! Source of record: qcb_audit/add3x_bench/qcb_add3x.rs. Compiled by copying it into a
//! scratch extraction of 60d61859 as src/bin/qcb_add3x.rs (that tree's lib exposes
//! `point_add`). No contestant file is edited. Usage: qcb_add3x [shots]

use alloy_primitives::U256;
use quantum_ecc::circuit::{analyze_ops, BitId, Op, OperationType, QubitId, QubitOrBit};
use quantum_ecc::point_add::trailmix_ludicrous::ec_add::{build_add3x_test, build_add3x_test_orig};
use quantum_ecc::sim::Simulator;
use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake256,
};

// Mirror the shipped configuration (incumbent mod.rs:1179-1198 and
// trailmix_ludicrous/mod.rs:430-451), minus TLM_CUCCARO_SKIP_STRUCTURAL_DEAD_CALLS,
// whose skip tables are keyed to call indices of the full circuit.
const ENV: &[(&str, &str)] = &[
    ("TLM_TARGET_Q", "1150"),
    ("TLM_FOLD_CHUNK_ZERO_CIN", "1"),
    ("TLM_FFG_MAX_G", "47"),
    ("TLM_APPLY_ADD_SKIP_LASTK", "1"),
    ("TLM_FOLD_TAIL_CINC", "1"),
    ("TLM_CODEC_DIAMOND_MCX", "1"),
    ("TLM_COORD_RSUB_FUSED", "1"),
    ("TLM_SQUARE_VENT_MARGIN", "0"),
    ("TLM_COORD_ADD3X_TRUNC", "1"),
    ("TLM_SQUARE_VENT_SHIFTED", "1"),
    ("TLM_SQUARE_PEAK_CAP", "1150"),
];

struct Res {
    ops: usize,
    qubits: u64,
    ccx_ccz: usize,
    avg_tof: f64,
    mismatches: usize,
    phase_bad: usize,
    anc_bad: usize,
    n: usize,
}

fn rand_below<R: XofReader>(xof: &mut R, p: U256) -> U256 {
    loop {
        let mut b = [0u8; 32];
        XofReader::read(xof, &mut b);
        let v = U256::from_le_bytes(b);
        if v < p {
            return v;
        }
    }
}

fn run(ops: &[Op], ox: &[BitId], dst: &[QubitId], shots: usize, p: U256, seed: &[u8]) -> Res {
    let (nq, nb, _, _) = analyze_ops(ops.iter());
    let ox_reg: Vec<QubitOrBit> = ox.iter().map(|&b| QubitOrBit::Bit(b)).collect();
    let dst_reg: Vec<QubitOrBit> = dst.iter().map(|&q| QubitOrBit::Qubit(q)).collect();
    let mut is_dst = vec![false; nq as usize];
    for q in dst {
        is_dst[q.0 as usize] = true;
    }

    let mut h = Shake256::default();
    h.update(seed);
    let mut inputs = h.finalize_xof();
    let mut h2 = Shake256::default();
    h2.update(b"qcb-add3x-measurement-rng");
    let mut rng = h2.finalize_xof();
    let mut sim = Simulator::new(nq as usize, nb as usize, &mut rng);

    let three = U256::from(3u8);
    let (mut mism, mut pbad, mut abad) = (0usize, 0usize, 0usize);
    let mut done = 0usize;
    while done < shots {
        let bs = 64.min(shots - done);
        let mask: u64 = if bs == 64 { u64::MAX } else { (1u64 << bs) - 1 };
        sim.clear_for_shot();
        let mut exp = Vec::with_capacity(bs);
        for s in 0..bs {
            let d = rand_below(&mut inputs, p);
            let o = rand_below(&mut inputs, p);
            sim.set_register(&dst_reg, d, s);
            sim.set_register(&ox_reg, o, s);
            exp.push(d.add_mod(o.mul_mod(three, p), p));
        }
        sim.apply_iter(ops.iter());
        for s in 0..bs {
            if sim.get_register(&dst_reg, s) != exp[s] {
                mism += 1;
            }
        }
        if sim.phase & mask != 0 {
            pbad += 1;
        }
        if (0..nq as usize).any(|q| !is_dst[q] && sim.qubits[q] & mask != 0) {
            abad += 1;
        }
        done += bs;
    }
    let ccx_ccz = ops.iter().filter(|o| matches!(o.kind, OperationType::CCX | OperationType::CCZ)).count();
    Res {
        ops: ops.len(),
        qubits: nq,
        ccx_ccz,
        avg_tof: sim.stats.toffoli_gates as f64 / shots as f64,
        mismatches: mism,
        phase_bad: pbad,
        anc_bad: abad,
        n: shots,
    }
}

fn main() {
    let shots: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(9024);
    for (k, v) in ENV {
        std::env::set_var(k, v);
    }
    let p = U256::from_str_radix("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();
    println!("env mirrored: {}", ENV.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" "));
    println!("shots per circuit: {shots} (random dst, ox < p; same inputs for both circuits)");
    let mut out = Vec::new();
    for (name, (ops, ox, dst)) in [("add3x (shipped, classical 3*ox)", build_add3x_test()), ("add3x_orig (quantum-compatible)", build_add3x_test_orig())] {
        let r = run(&ops, &ox, &dst, shots, p, b"qcb-add3x-inputs");
        println!(
            "{name:<34} ops={:>7} qubits={:>4} CCX+CCZ emitted={:>5} avg executed Toffoli={:>9.3} | mismatches={} phase-bad batches={} ancilla-bad batches={} (n={})",
            r.ops, r.qubits, r.ccx_ccz, r.avg_tof, r.mismatches, r.phase_bad, r.anc_bad, r.n
        );
        out.push(r);
    }
    println!("delta (orig - shipped) avg executed Toffoli: {:+.3}", out[1].avg_tof - out[0].avg_tof);
}
