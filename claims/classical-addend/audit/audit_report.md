# QCB audit report: classical-addend interface gap (ecdsa.fail)

| | |
|---|---|
| Commit audited | `3dee0cd67d09d40af62da06bc1c1c6b16adc8f56` (HEAD of `main`, "Validate submission 54fc4f25-…") |
| Date | 2026-10-19 |
| Tool / model | Claude Code (desktop app, Code tab), Claude Opus 5.5 (`claude-opus-5-5`) |
| Method | V1: read-only, no builds or runs. V2: source reading plus measured runs on scratch extractions of the audited commits (see V2 "Method"). No tracked files modified. |
| Status | **V1–V4, review follow-up (Q1–Q4) and critique pass done (V4 from source and metadata only). Stopped for human review; V5 not started.** Kill flag **K3 HOLDS** (see V3). Build proposals P1/P2 were approved and run; their results are folded into V3, Q1, Q4 and C2. |

All `path:line` citations are on HEAD unless a commit is given. `INFERRED` marks anything I did not read directly in code or measure.

## Harness files and when they last changed

| File | Lines | Last commit touching it | Role |
|---|---|---|---|
| `src/circuit.rs` | 389 | `fa473ea9` 2026-05-28 | Op format, `validate()`, `analyze_ops()` (registers, qubit count) |
| `src/sim.rs` | 244 | `fa473ea9` 2026-05-28 | Bit-sliced simulator and gate counters |
| `src/bin/eval_circuit.rs` | 551 | `c66c9586` 2026-06-22 | Trusted scorer: seeding, test inputs, checks, averaging, `score.json` |
| `src/bin/build_circuit.rs` | 110 | `aa32e1c9` 2026-08-10 | Untrusted stage: calls `point_add::build()`, writes `ops.bin` |
| `src/lib.rs` | 28 | `aa32e1c9` 2026-08-10 | Exposes `circuit`, `sim`, `weierstrass_elliptic_curve` only |
| `src/weierstrass_elliptic_curve.rs` | 136 | `3bc1a61b` 2026-04-23 | Reference adder used to compute expected outputs |

`src/main.rs` does not exist at HEAD (see conflict C1). `aa32e1c9` touched only `build_circuit.rs` and `lib.rs`, so the simulator and scorer logic below has been unchanged since 2026-06-22.

---

## V1: Mechanism

### Answer table

| # | Question | Answer | Evidence |
|---|---|---|---|
| a | How are `target_x`, `target_y`, `offset_x`, `offset_y` represented? Where are offsets loaded? | Registers are identified by position, not by name. The scorer requires exactly 4 registers of width 256. Registers 0 and 1 must be all qubits; **registers 2 and 3 (the offsets) must be all classical bits**. Before each 64-shot batch the simulator zeroes everything, then writes target (x,y) into qubits and offset (x,y) into bits, one shot per bit-lane. | `eval_circuit.rs:471-508` (shape/type checks); `eval_circuit.rs:286-293` (per-batch load); `sim.rs:61-69` (`clear_for_shot`); `sim.rs:192-217` (`set_register`); `circuit.rs:370-383` (registers built from `AppendToRegister` ops by `r_target`) |
| b | How are classically conditioned ops encoded and executed? | Every `Op` has a `c_condition: BitId` field (text form `… if bN`). There is also a condition stack (`PushCondition` / `PopCondition`). For each op the simulator computes `cond = stack_AND & bit(c_condition)`, a 64-lane mask, and ANDs every gate effect with it. `c_condition` is allowed on CCX, CCZ, CX, CZ, Swap, X, Z, R, Neg, Hmr, BitInvert, BitStore0, BitStore1. | `circuit.rs:104-113` (fields); `circuit.rs:276-282` (text syntax); `eval_circuit.rs:134` (binary field); `circuit.rs:164-193` (where allowed); `sim.rs:73-80` (mask); `sim.rs:101-176` (masked effects); `sim.rs:168-176` (stack) |
| c | How is the Toffoli count computed? Are condition-false Toffolis excluded? Averaged? | Each CCX or CCZ adds `popcount(cond)`, the number of shots where its classical condition is true. **Condition-false shots are not counted.** Quantum control values do not matter: a CCX with \|0⟩ controls still counts. The reported T is total / n, where n is the number of test points kept (≤ 9024). Score = round(T) × Q, where Q = highest qubit index used + 1. | `sim.rs:82-87` (count); `sim.rs:88-98` (Clifford set; X/Z uncounted); `eval_circuit.rs:357-361` (average); `eval_circuit.rs:432-436, 548` (score); `circuit.rs:355-363` (Q) |
| d | Is there uncounted computation on classical bits? Can measurement outcomes feed conditions? | **Yes to both.** `BitInvert`, `BitStore0`, `BitStore1` count as neither Toffoli nor Clifford. Combined with the condition stack, they give a free classical Toffoli: `PushCondition b1; BitInvert b3 if b2; PopCondition` sets b3 ^= b1·b2. So any Boolean function of the offset bits can be computed at zero T and zero Q. `Hmr` writes a measurement outcome into a classical bit, and that bit can then be used as a condition. | `sim.rs:156-164` (bit ops); `sim.rs:84-99` (not counted); `sim.rs:168-171` + `sim.rs:77-80` (AND via the stack); `sim.rs:140-148` (Hmr → bit); `circuit.rs:182-186` (Hmr has `c_target`) |
| e | How are test inputs and measurement randomness seeded? | A single SHAKE256 XOF over the whole op stream, with domain tag `quantum_ecc-fiat-shamir-v2`. For each of 9024 attempts it reads k1, k2 (32 bytes each, little-endian) and sets t = k1·G, o = k2·G. It skips without redrawing if t.x == o.x, t = O, or o = O. **The same XOF stream** then drives the simulator: every R/Hmr execution reads 8 bytes. All randomness is therefore a deterministic function of `ops.bin`. | `eval_circuit.rs:204-218` (seed); `eval_circuit.rs:45, 248-270` (inputs and filters); `eval_circuit.rs:273` (XOF passed to simulator); `sim.rs:141-143, 150-152` (8 bytes per R/Hmr) |

### Kill-criteria status after V1

- **K1 (offsets not classical / not usable as conditions): NOT triggered.** The scorer enforces that registers 2 and 3 are classical bits (`eval_circuit.rs:499-508`). Any `BitId` may be a `c_condition` (`circuit.rs:111`, `sim.rs:78-80`) or be pushed onto the condition stack (`sim.rs:168-171`).
- **K2 (conditioned Toffolis counted regardless of condition): NOT triggered.** The counter adds `cond.count_ones()` (`sim.rs:82, 86`), where `cond` already includes the classical condition.
- **K3:** not assessed yet (needs V3).

### Detail

#### a. Register representation and per-shot loading

- The scorer never sees register names. `analyze_ops` builds registers from `AppendToRegister` ops, indexed by `r_target` (`circuit.rs:370-383`). The checks are positional (`eval_circuit.rs:471-508`).
- In HEAD contestant code the four registers are `x`, `y` (qubits) and `ox`, `oy` (bits). They are allocated in that order (`src/point_add/mod.rs:228-231`) and declared at the end of the stream (`mod.rs:305-308`; Leapfrog path `mod.rs:263-266`). Each declaration emits `AppendToRegister` ops plus a `Register` op with the next register id (`src/point_add/builder.rs:401-417`). Declaring them at the end works because `analyze_ops` is a static pre-pass (`eval_circuit.rs:469`) that runs before the inputs are loaded (`eval_circuit.rs:287-293`).
- **"Classical" means per-shot classical.** The simulator is bit-sliced: each qubit and each bit is a `u64`, and bit i is shot i (`sim.rs:15-23`, `sim.rs:203-213`). An offset bit can be 1 in some lanes and 0 in others. Conditioning is applied lane by lane, and so is counting (`popcount`).
- The offsets are never checked after the run. The output check reads only registers 0 and 1 (`eval_circuit.rs:297-311`). The cleanliness check zeroes the register *qubits* and then scans qubits only (`eval_circuit.rs:326-340`). Classical bits are not checked at all, so offset bits and scratch bits may be left modified.

#### b. Encoding and execution of classical conditions

- There are two mechanisms, and they compose:
  1. the per-op `c_condition` field (`circuit.rs:111`);
  2. a condition stack. `PushCondition` saves the current base mask and ANDs a bit into it (`sim.rs:168-171`). `PopCondition` restores the saved mask (`sim.rs:172-176`). Popping an empty stack is silently ignored (`sim.rs:173`).
- The base mask starts as all ones (`sim.rs:74`), and the stack is local to each `apply_iter` call (`sim.rs:73`).
- Every effect is masked by `cond`. For example, CCX does `target ^= cond & c1 & c2` (`sim.rs:103-104`), X does `target ^= cond` (`sim.rs:119-121`), and Hmr/R act only on `cond` lanes (`sim.rs:144-154`).

#### c. Toffoli counting and averaging

- `executed_shots = cond.count_ones()` (`sim.rs:82`). CCZ and CCX add it to `toffoli_gates` (`sim.rs:85-87`). CX, CZ, Swap, R and Hmr add it to `clifford_gates` (`sim.rs:88-93`). X, Z, Neg, the Bit ops, Push/Pop and the register ops are not counted (`sim.rs:95-98`). The Clifford count is not part of the score.
- T = `toffoli_gates / n`, where n = `targets.len()` after filtering (`eval_circuit.rs:271, 357-361`). The score is `round(T) × total_qubits` (`eval_circuit.rs:434-435`).
- **Latent off-by-lanes (INFERRED to be dormant).** The base mask is all ones on all 64 lanes, even in a final partial batch (`sim.rs:74`; `eval_circuit.rs:283-284`). If any point were filtered out (n < 9024), the dead lanes of the last batch would still count unconditioned Toffolis while the denominator stays n. This could only *over*-count, never give a discount. 9024 = 141 × 64, and a filter hits with probability ≈ 2⁻²⁵⁵ per draw, so in practice n = 9024 and every batch is full. (Not measured.)
- **What this means for the hypothesis.** A Toffoli whose control is an addend bit can be emitted as `CX … if ox[i]` (`circuit.rs:170-174`), which costs 0 T (Clifford only). Or it can stay a CCX conditioned on `ox[i]`, which costs about 0.5 T per shot on average because offset bits are about half ones (INFERRED from the input distribution; not measured). Writing A into qubits with `X q if ox[i]` (`circuit.rs:175-178`) is completely free (`sim.rs:95-98`). **Hypotheses (a) and (c) of the brief are confirmed as mechanisms.** Whether and how much the incumbent uses them is V2.

#### d. Uncounted classical computation and measurement-fed conditions

- **Free classical logic.** BitInvert, BitStore0 and BitStore1 accept a condition and act on any bit (`circuit.rs:187-190`, `sim.rs:156-164`). With the stack, they make a classical Toffoli at zero cost (table row d). Nested pushes give multi-input AND. Classical bits are not counted in Q, which counts only qubit ids (`circuit.rs:355-363`); `num_bits` is unused in scoring (`eval_circuit.rs:356`). Classical scratch never has to be cleaned (see a). So classical precomputation on A is free in all three scored resources (T, Q, correctness checks), and the op count is unscored.
- **Measurement outcomes.** `Hmr` (X-basis measurement + reset) stores a random outcome `m` in `c_target`, XORs `qubit & m` into the global phase, and zeroes the qubit (`sim.rs:140-148`). The outcome is a fresh XOF value, independent of the qubit's value. That is correct for an X-basis measurement of a computational-basis state. Using `m` as a condition for a phase fix-up (e.g. `CZ a b if m`) is the measurement-based-uncomputation pattern. Such a fix-up is counted only on shots where m = 1 (about 50%). **This is legitimate physics and is not an interface artifact.** V2 must keep measurement-derived conditions separate from offset-derived ones.
- **Data flow into classical bits.** No op copies a qubit's computational value into a bit; Hmr's outcome does not depend on the state. So a classical bit can depend only on (i) offset inputs, (ii) measurement randomness, and (iii) constants. That makes "conditioned on a bit derived from the offsets" well defined and traceable by a simple taint pass over the Bit ops and conditions. This is the plan for the V2 instrument.
- **Correction to hypothesis (b).** `point_add::build()` takes no arguments (`mod.rs:503`; called at `build_circuit.rs:92`), and the offsets are drawn from a hash of the *finished* op stream (`eval_circuit.rs:514-515`). Nothing about A is known at build time, so a value like 3·x_A mod p **cannot be precomputed at build time**. It can still be had for free in T and Q, but only at run time: computed in-circuit from `ox` with uncounted Bit ops, or avoided by conditioning quantum adders directly on `ox` bits. Which route the incumbent takes (HEAD has a `coord_add3x(circ, x, ox)` phase at `mod.rs:285-286`, and on the Leapfrog path at `mod.rs:245-246`) is a V2 question.

#### e. Seeding

- Seed: SHAKE256 over `"quantum_ecc-fiat-shamir-v2" ‖ u64le(op count) ‖ for each op: kind as u8 ‖ 6 × u64le fields` (`eval_circuit.rs:204-218`). It is computed once over the loaded ops (`eval_circuit.rs:514`).
- Inputs: k1 and k2 are uniform 256-bit values, not reduced mod n before scalar multiplication (`eval_circuit.rs:249-256`; `weierstrass_elliptic_curve.rs:123-135`). Expected output = reference `curve.add` (`eval_circuit.rs:266`).
- **The filters remove all exceptional cases from the test set** (`eval_circuit.rs:257-265`). A = O never occurs, and neither do R = O, R = A (doubling) or R = −A. A circuit is never tested on the A = O row that a windowed table would contain. This matters for V3 item 3 and for the "random offset masks" objection.
- Measurement randomness: the same XOF reader continues into the simulator (`eval_circuit.rs:273`). Every R or Hmr op reads 8 bytes *per batch, even on lanes where its condition is false* (`sim.rs:141-143, 150-152`).
- **The seed can be ground.** X ops change the hash but not T (`sim.rs:95-98`). HEAD contestant code appends 96 X ops and sets their targets to q0/q1 pairs that encode a 48-bit `TAIL_NONCE` (`mod.rs:200-214`, `mod.rs:623-628`). The nonce is pinned to `1050066314463` with the comment "all 9024 Fiat-Shamir shots pass" (`mod.rs:155-156`). INFERRED: other nonces fail some shots, so the HEAD circuit is not exact on all inputs and its test set was selected by grinding. This matches the description in teammate claim 1 (`qcb_audit/context/robust_architecture_specification.md` §1.1). For this audit it means **any T measured in V2 or V3 is specific to the circuit's nonce-selected test set.** A Toffoli count that depends on offset bits will be close to, but not exactly, its expectation over random A.

### Conflicts with the brief / README (the code wins)

- **C1. `src/main.rs` is not a contract file at HEAD; it does not exist.** It was removed in `fa473ea9` (2026-05-28, "seperate and benchmark"). The harness is now split:
  - `src/bin/build_circuit.rs` (untrusted; runs contestant code);
  - `src/bin/eval_circuit.rs` (trusted scorer);
  - `src/lib.rs`, `circuit.rs`, `sim.rs`, `weierstrass_elliptic_curve.rs`.

  `benchmark.sh:5-16` runs the two binaries in sequence. The README's contract list and its single `cargo run --release -- --note …` command predate the split. INFERRED: with two `[[bin]]` targets and no `default-run` (`Cargo.toml:9-15`), a bare `cargo run` would be ambiguous. Not run.
- **C2. Q is not "peak live qubits" (brief glossary).** The code computes Q as the highest qubit index referenced by any op, plus 1 (`circuit.rs:355-363`, `eval_circuit.rs:469, 548`). This is static: nothing tracks liveness. It equals peak live qubits only if the builder recycles indices compactly (INFERRED).
- **C3. The README's "forward∘reverse identity" and "sim.rs enforces [cleanliness] on every freed qubit" checks do not exist at HEAD.** The scorer runs one forward pass (`eval_circuit.rs:295`). It then checks outputs, phase, and the final state of non-register qubits (`eval_circuit.rs:297-353`). `sim.rs` has no allocate/free tracking. Commit `1447f8b8` ("sim.rs: promote R to a hard assertion") was followed by `b15b7c9a` ("revert sim.rs and circuit.rs to upstream verbatim"), which removed 43 lines net from `sim.rs`. INFERRED from commit messages and `--stat`; not diffed.
- **C4. A scorer comment contradicts the validator.** The comment at `eval_circuit.rs:64-65` says `validate()` rejects "R/Hmr with c_condition". But `circuit.rs:175-186` marks `c_condition` as ALLOWED for both R and Hmr. Also, the condition stack applies to R/Hmr regardless (`sim.rs:77`). Effect: a conditioned reset leaves the qubit dirty on condition-false lanes. If that is never cleaned, the final ancilla check still catches it (INFERRED), so this is not a scoring hole by itself. Out of scope for this claim; noted for completeness.
- **C5. Hypothesis (b) needs rewording.** "Precompute addend-derived constants for free" is true in cost, but the mechanism is run-time classical-bit computation, not build-time precomputation (see d).
- **C6. The test set never contains A = O or R = ±A** (`eval_circuit.rs:257-265`). The brief's framing of the A = O conditioning cost (V3 item 3) should note that the leaderboard interface never requires it.

### Teammate-spec identifiers checked in passing (§6 of the brief)

- **Exist in code:** `OperationType`, `Op.{q_control1, q_control2, q_target, c_condition}`, `Simulator::new(num_qubits, num_bits, &mut xof)`, `clear_for_shot`, `set_register` / `get_register`, `sim.phase`, `cond_mask`, `NUM_TESTS = 9024`, `fiat_shamir_seed`.
- **Do not exist in the harness** (`grep` over `src/*.rs src/bin/*.rs` returned nothing): `Op::adjoint`, `is_abi_register`, any canonicalizer. The reference code in `robust_architecture_specification.md` §4 would not compile against this harness as written.
- That spec says `eval_circuit.rs` is "552 lines"; it is 551 at HEAD.
- Its description of the 96-X / 48-bit `TAIL_NONCE` matches `mod.rs:200-214, 623-628`. Its claim that circuits "with failure rates up to 15%" passed validation is **not verified**.

### INFERRED items (V1)

1. n = 9024 in practice (filter probability ≈ 2⁻²⁵⁵ per draw), so the dead-lane over-count in partial batches is dormant.
2. A Toffoli conditioned on one offset bit averages about 0.5 counted per shot (offset bits ≈ Bernoulli(½)).
3. The pinned `TAIL_NONCE` implies the HEAD circuit fails some shots under other nonces.
4. The README's `cargo run` instruction is stale after the binary split.
5. C2 (Q equals peak live only with compact index reuse) and C4 (a dirty conditioned reset is caught by the final check).

### Open issues and prerequisites for V2

- **Harness version vs. incumbent.** *Resolved in V2:* `git diff 60d61859 HEAD` and `git diff 897dda2b HEAD` on `sim.rs`, `circuit.rs`, `eval_circuit.rs` and `weierstrass_elliptic_curve.rs` are empty, so V1 applies to both audited circuits.
- **Instrumented count needs an op stream.** Plan: a new bin under `qcb_audit/`, using only the existing crate with no new dependencies. It would:
  1. load `ops.bin` exactly as `eval_circuit.rs:96-168` does;
  2. taint bits: offset registers → `offset`, Hmr outputs → `meas`, then propagate through Bit ops and condition-stack context;
  3. re-simulate with Toffoli counters split into unconditioned / offset-conditioned / measurement-conditioned / mixed.

  Producing `ops.bin` means running `build_circuit`, which executes contestant code. On Windows, `benchmark.sh` would fall back to an *unconfined* run (`benchmark.sh:10-11`). Per brief §0.5 I will ask before building.
- **Memory risk.** `Op` is 56 bytes in memory and the scorer holds the whole stream (`eval_circuit.rs:121`). The baseline circuit has about 28.7 M ops (`results.tsv` row `fa473ea`), so about 1.6 GB. The incumbent's op count is unknown. If it is above about 140 M ops, a run would exceed the brief's 8 GB threshold. Check the header op count first (`eval_circuit.rs:101-107`), or stream ops instead of loading them all. *Resolved in V2:* the incumbent has 9.1 M ops, and every run peaked under 1.5 GB.

---

## V2: Exposure in the incumbent

### Headline

**Neither audited circuit has a single Toffoli whose classical condition depends on the addend.** Measured on the official test inputs:

- **0** CCX/CCZ are offset-conditioned in the incumbent, and **0** in 3616dbf2.
- All classically conditioned Toffolis are conditioned on measurement (HMR) outcomes. That is legitimate measurement-based uncomputation, and they fire on 50.0% of shots.

The addend gets in by three routes only:

1. classically conditioned X loads into a temp qubit register (free on the leaderboard);
2. **free classical precomputation of 3·x_A mod p and x_A + 1**;
3. generic quantum-quantum modular adders on that temp register.

The A-touching arithmetic costs **1,600 T/shot** in both circuits. That is 0.12% of T for the incumbent and 0.17% for 3616dbf2, and it would cost the same with A held in qubits.

What would not carry over to a windowed setting is the *loading* (10 × 256 free conditioned X) and the free 3·x_A / x_A + 1. That points V3 toward reading **R2**, with K3 likely. This is not yet decided: see "Pointers for V3".

### Sources located

| Item | Value | Evidence |
|---|---|---|
| "8e9c9a2" | A **submission UUID prefix, not a commit**: `8e9c9a22-a6ec-4950-8c5b-c12aade0bc8e` | `git cat-file -t 8e9c9a2` fails; `git log --all` finds it only in commit subjects |
| Incumbent source | `60d61859fa6965eba53634f91877b1141a6f9dce` "Accept submission 8e9c9a22-…", 2026-07-26, on `main`. Same tree (`9c633873…`) as `ae7676dd`, the head of `origin/submissions/8e9c9a22-…` | `git rev-parse 60d61859^{tree} ae7676dd^{tree}` |
| "3616dbf" | Submission UUID prefix `3616dbf2-f32d-41c8-9ea2-7a5bbe372fc0` | as above |
| 3616dbf2 source | `897dda2b0cf267151ecd973252d2a5078cbf1b63` "Validate submission 3616dbf2-…", 2026-08-21. It is the head of `origin/submissions/3616dbf2-…`, is on `main`, and has `60d61859` as an ancestor | `git log origin/submissions/3616dbf2-…`; `git merge-base --is-ancestor` |
| Harness at both commits | Identical to HEAD (see V1 open issue, now resolved) | `git diff --stat` empty |

`src/point_add/` paths below are relative to the commit named. "inc" = `60d61859`, "pp" = `897dda2b`.

### Method

1. **Sources.** Each commit was extracted with `git archive <commit> | tar -x` into the session scratchpad, outside the repo.
2. **Build.** `cargo build --release --locked --offline`.
   - Toolchain: the already-installed **stable rustc/cargo 1.99.1**, forced with `RUSTUP_TOOLCHAIN=stable`. This is *not* the pinned 1.93.0 (see "Process notes").
   - Crates: fetched once from crates.io as pinned by `Cargo.lock` (192 crates). All three trees' locks are byte-identical to HEAD's.
3. **Run.** `build_circuit` (contestant code) ran **unsandboxed**, with your approval: on Windows, `benchmark.sh`'s bubblewrap/sandbox-exec are unavailable. It ran in a scratch directory under a watchdog with an 8 GB kill limit. Then the **official `eval_circuit`** ran on the resulting `ops.bin`; it writes `score.json`/`results.tsv` into the scratch copy, not the repo.
4. **Fidelity check.** On the incumbent, the official scorer reproduces the paper's numbers exactly: **T = 1,299,452.699 → 1,299,453, Q = 1,151**, all 9,024 shots OK (`qcb_audit/results/v2_60d61859_official_eval.txt`). The newer compiler therefore did not change the op stream in any way that matters.
5. **Instrument.** `qcb_audit/taint_count/` is a new, standalone crate. It depends only on the harness library (`quantum_ecc` = circuit/sim/curve, no contestant code) and the existing crates.
   - **Static pass.** A single forward pass over the straight-line op stream. Classical bits carry the taint set {OFF = from `offset_x`/`offset_y`, MEAS = from an HMR outcome}. Taint propagates through BitInvert/BitStore*/HMR and the condition stack.
     - Each CCX/CCZ gets a **condition class**: the taint of `c_condition` together with every bit on the condition stack.
     - Each CCX/CCZ also gets a **control class**: whether a control qubit is still "classical-known", meaning its value depends only on classical bits, and carries OFF data. Such a Toffoli could have been a free `CX if bit` on this interface.
   - **Dynamic pass.** Replays the official Fiat-Shamir inputs and the HMR/R random stream, using code copied from `eval_circuit.rs`. It re-implements `sim.rs` `apply_iter` with a per-op executed-Toffoli counter, and on **every batch asserts identical qubits, bits, phase and Toffoli total against `quantum_ecc::sim::Simulator`**. All 141 batches matched, so the per-class numbers are exact on the official test set.
   - **Limitation.** The control-class test only catches Toffolis whose control is still a *pure* classical copy of A. Once an adder mixes A with quantum data (e.g. a MAJ step), the control counts as quantum. So it finds trivially convertible sites, not every possible redesign. The redesign headroom is bounded analytically in (b).
   - Raw outputs: `qcb_audit/results/v2_incumbent_60d61859_taint.txt`, `qcb_audit/results/v2_3616dbf2_897dda2b_taint.txt`.

| Run | Wall time | Peak RAM |
|---|---|---|
| inc `build_circuit` | 23 s | 1.47 GB |
| inc `eval_circuit` | 26 s | 0.50 GB |
| inc instrument | 40 s | 0.60 GB |
| pp `build_circuit` | 4 s | 0.74 GB |
| pp `eval_circuit` | 34 s | 0.74 GB |
| pp instrument | 72 s | 0.89 GB |

### Answer table

| # | Question | Incumbent `60d61859` (8e9c9a22) | 3616dbf2 `897dda2b` | Evidence |
|---|---|---|---|---|
| a | CCX/CCZ ops whose execution depends on an offset bit (directly, or via an offset-derived classical bit) | **0** of 1,358,316 (1,353,116 CCX + 5,200 CCZ) | **0** of 1,009,077 (1,009,049 CCX + 28 CCZ) | measured, static pass; taint files "cond class OFF" and "OFF+MEAS" |
| a′ | For contrast: CCX/CCZ conditioned on HMR outcomes | 117,747 | 112,719 | same |
| a″ | CCX/CCZ with a classical-known *control* carrying A data (trivially convertible to `CX if bit`) | 5: the first carry AND of each A-loaded adder | 5: same sites | same; op list at the end of each taint file |
| b | Executed Toffolis/shot that depend on offset-derived conditions | **0.000** | **0.000** | measured, dynamic pass, cross-checked per batch |
| b′ | Executed, HMR-conditioned | 58,883.699 (50.01% of 117,747) | 56,349.477 (49.99% of 112,719) | same |
| b″ | Executed, unconditioned | 1,240,569.000 (every one fires on every shot) | 896,358.000 | same |
| b‴ | Executed Toffolis/shot inside the five A-loaded adders | **1,600.1** (0.12% of T) | **1,600.0** (0.17% of T) | measured, per-window sums (table below) |
| b⁗ | Quantum-A equivalent of the arithmetic | Same 1,600 T: the adders already take A from qubits. Converting the 5 sites in a″ would *save* 5 T on the leaderboard. Redesign headroom for a classical-specialized adder: about 10²–10³ T/shot (INFERRED, see (b)) | same | INFERRED |
| c | Addend-derived values computed classically | 3·x_A mod p and x_A + 1 mod 2²⁵⁶, using uncounted bit ops (23,397 OFF-dependent bit/push ops, including the 2,560 load pushes). Raw x_A and y_A are loaded as-is | identical code | `ec_add.rs` (inc) `84-154`, `217-229`; measured op counts |
| d | 3616dbf2 | Same coordinate shell. Only the two division callbacks change (ping-pong GCD) | — | pp `pingpong_div.rs:1172-1200`, pp `ec_add.rs:298-333` |

T totals: inc **1,299,452.699**, Q 1,151, score 1,495,670,403. pp **952,707.477**, Q 1,321, score 1,258,525,947. All 9,024 shots OK for both, in the official scorer and in the instrument. The pp numbers are measured here; the brief gives none to compare against.

### a. Where the offset bits go (source, confirmed by measurement)

- `ox`/`oy` are allocated as bits and passed only to `ec_add` (inc `trailmix_ludicrous/mod.rs:460-465`). `ec_add` has exactly five offset sites, each inside a `set_phase` (inc `trailmix_ludicrous/ec_add.rs:287-309`):

| # | Phase (inc `ec_add.rs` line) | Payload put into a fresh 256-qubit temp register | Adder | Load/unload runs in the op stream | Executed T in window, inc / pp |
|---|---|---|---|---|---|
| 1 | `tlm_coord_x_sub` (288) | x_A | `mod_sub_vented` (line 44) | runs 0 / 1 | 320.1 / 319.9 |
| 2 | `tlm_coord_y_sub` (290) | y_A | `mod_sub_vented` | runs 2 / 3 | 320.0 / 320.0 |
| — | `tlm_inverse` (292-294) | — | GCD division | between runs 3 and 4 | 618,725.1 / 445,895.4 |
| 3 | `tlm_coord_add3x` (297) | **3·x_A mod p, computed classically** (lines 60, 84-154) | `mod_add` (line 68; `TLM_COORD_ADD3X_TRUNC=1`, inc `mod.rs:1194`) | runs 4 / 5 | 319.8 / 320.1 |
| — | `tlm_square` + `tlm_forward_multiply` (299-304) | — | square + division run forward | between runs 5 and 6 | 679,127.6 / 505,212.1 |
| 4 | `tlm_coord_y_sub_final` (307) | y_A | `mod_sub_vented` | runs 6 / 7 | 320.2 / 319.9 |
| 5 | `tlm_coord_rsub_final` (309) | **x_A + 1 mod 2²⁵⁶, computed classically** (lines 236, 217-229; `TLM_COORD_RSUB_FUSED=1`, inc `mod.rs:1192`) | `mod_rsub_vented_loaded` (line 241) | runs 8 / 9 | 320.0 / 320.1 |

- Mapping runs to phases is INFERRED from order and Toffoli volume. It is consistent: the windows sum to T within rounding (inc: 1,600.1 + 618,725.1 + 679,127.6 = 1,299,452.8). There are 0 Toffolis between runs 1 and 2 and between runs 7 and 8.
- **Each load and unload is 256 × (`PushCondition b; X q; PopCondition`).** That is `x_if_bit` (inc `trailmix_ludicrous/mod.rs:81-85`). The instrument counts 2,560 offset-conditioned X = 10 runs × 256. These X gates are uncounted (V1 c), so on the leaderboard **loading A costs 0 T and 0 Clifford**.
- **Every other classical condition in the incumbent is an HMR outcome.** The only builder sites that call `push_condition` (other than `x_if_bit`) or set `c_condition` directly do so right after `hmr(q, bit)`:
  - inc `trailmix_ludicrous/arith.rs:644, 683, 1135, 1464, 1491`;
  - inc `gidney.rs:1361, 1445`; inc `gcd.rs:1858`; inc `fused.rs:1313`; inc `comparator.rs:910`;
  - inc `arith/compare.rs:33-263`; inc `venting.rs:195, 318-336, 636, 739`.

  Measurement confirms it: condition class OFF = 0.
- **The constant-propagation pass is sound with respect to A.** Offset bits are not treated as known constants. A conditioned X on a not-provably-true condition makes the qubit Unknown (inc `trailmix_ludicrous/constprop.rs:61-76, 170-176`). So no gate is specialized on A's value at build time, which is also impossible because A is drawn after the build (V1 d).
- **A quantum-compatible variant of the 3x step sits unused in the code:** `coord_add3x_orig` (inc `ec_add.rs:336-351`). It loads x_A once and does `mod_add`, `mod_double`, `mod_add`, `mod_double_reverse`. It is not on the build path.

#### b. Executed cost and the quantum-A equivalent, per site

- **Arithmetic: no change.** The five adders operate on a temp *qubit* register, so with A held in qubits they cost the same: 1,600 T/shot measured.
  - The 5 "classical-OFF control" Toffolis are the first carry AND of each adder. Examples: inc op 1535 `CCX q512(temp[0]) q0(x[0])`; op 8143 `CCX q767 q256(y[0])`. They are the only places where the leaderboard interface would let a contestant *save* Toffolis by swapping a CCX for `CX if ox[i]`: **5 T/shot**, and the incumbent doesn't take that saving.
  - Redesign headroom (INFERRED): a modular add specialized to a classical addend could at best approach the n − 1 = 255 floor for an uncontrolled add. That floor holds for both classical and quantum addends, per the contestants' own analysis (inc `memory/03-proven-floors.md:32-33`; not verified). So the gap to the measured 320 is at most about 5 × 65 ≈ 325 T/shot. It would be more only if the modular-reduction step were itself specialized; order 10²–10³ T in total.
- **Loading (10 runs × 256 conditioned X, 0 T now).** With quantum A this is the lookup cost. Rough per-site options (INFERRED; numbers are standard literature, not measured here):
  - **Hold A in qubits for the whole addition:** each load/unload becomes 256 CX (0 T), at +512 qubits for x_A and y_A. 3·x_A and x_A + 1 then need computing in-circuit (see c) or another +512 qubits.
  - **One QROM lookup per load, A not held:** one table traversal per load (about 2¹⁶ T for w = 16). Each unload is a reverse traversal (about 2¹⁶ T) or a measurement-based unlookup (much cheaper, on the order of √L ≈ 2⁸ T). In the incumbent, loads 1–2 are back to back, as are 4–5, so a natural port has **three lookup stages**: {x_A, y_A}, {3·x_A mod p}, {y_A, x_A + 1}.

#### c. Addend-derived classical values

| Value | How it is computed | How it enters the circuit | Windowed equivalent (INFERRED) |
|---|---|---|---|
| 3·x_A mod p | Classical ripple adds of x_A three times, then a fold by 2²⁵⁶ ≡ 2³² + 977 and a ≥p select. Uses `bit_and_xor_into` / `bit_xor_into` / `bit_copy` = nested `PushCondition` + `BitInvert`/`BitStore1` (inc `ec_add.rs:84-154`, inc `mod.rs:720-739`). All uncounted | `x_if_bit` into temp, `mod_add`, `x_if_bit` unload; bits cleared with `bit_store0` (inc `ec_add.rs:62-81`) | Store 3·x_A mod p as the payload of that stage's lookup (0 extra traversal), or compute it from quantum x_A as in `coord_add3x_orig` (about +2 adder-class operations, roughly +0.6k–1k T at 320 T per modular add) |
| x_A + 1 mod 2²⁵⁶ | Classical increment (inc `ec_add.rs:217-229`), used by the fused reverse-subtract `x ← x_A − x` | `x_if_bit` into temp, `mod_rsub_vented_loaded`, unload (inc `ec_add.rs:235-251`) | Table payload (0 extra traversal), or a quantum increment (about n T) |
| x_A, y_A | — (raw input bits) | `x_if_bit` into temp (runs 0, 2, 6) | Table payload |

The instrument finds 0 non-register bits still OFF-tainted at the end of the stream, so all classical scratch derived from A is cleared, even though nothing requires it. The incumbent uses 525,426 classical bits in total and 3616dbf2 uses 1,010,987; nearly all are HMR outcomes and none are scored.

#### d. 3616dbf2 (post-cutoff, ping-pong GCD)

- **Route.** The default build takes the ping-pong path unless `SUB4_LEGACY_POINT_ADD` is set (pp `mod.rs:2525-2541`). `build_pingpong_point_add` reuses "the existing TrailMix coordinate shell … verbatim. Only the two division callbacks differ" (pp `pingpong_div.rs:1172-1200`). The shell is pp `ec_add.rs:298-333`; `ec_add.rs` differs from inc only by the `ec_add_with_division` refactor (`git diff --ignore-all-space --ignore-cr-at-eol`: +26/−2).
- **Same flags.** `TLM_COORD_RSUB_FUSED=1` and `TLM_COORD_ADD3X_TRUNC=1` are still set (pp `mod.rs:1198, 1200`). Every condition source in `pingpong_div.rs` is an HMR outcome (pp `pingpong_div.rs:438, 588, 668, 720, 836, 845, 916, 1026, 1056`).
- **Measured.** Same picture as the incumbent: 0 offset-conditioned Toffolis, the same 10 load runs and 2,560 conditioned X, the same 23,397 OFF-dependent classical ops, and 1,600.0 T in the five adders.
- **One difference.** 17 Toffolis/shot have a classical-*constant* control: removable dead weight, not A-related. The ping-pong route returns before the incumbent's post-passes (fan-out, constant propagation, census strip) are applied (pp `mod.rs:2525-2541`).
- **Nonce.** It also grinds a tail nonce (`SUB4_PINGPONG_TAIL_NONCE`, default 82; pp `mod.rs:2531-2534`).

### Pointers for V3 (INFERRED; nothing below is verified yet)

- **K3 looks likely for the incumbent.** The specialization it actually uses is (i) free loads and (ii) free 3·x_A / x_A + 1. Both can be reproduced in a windowed setting through the table payload at no extra traversal. The arithmetic-level headroom is at most about 10³ T. So part 4 of V3 (the residual specialization delta) should come out near 0 relative to +384,708, unless the windowed variant's arithmetic differs from the incumbent's.
- **The incumbent's structure predicts the paper's overhead almost exactly.** Its five load sites fall naturally into 3 stages, and the paper (§5.2.3, not verified) describes the variant as loading A "in 3 stages".
  - 3 lookups ≈ 3·2¹⁶ = 196,608 T, which is exactly the brief's "standard analytic accounting" line.
  - If the variant clears each payload by "exact reversible replay", i.e. one full reverse traversal per stage, that is another 3 traversals: 6 × 2¹⁶ = **393,216 T, within 2.2% of the measured +384,708** (paper).
  - The +188,100 excess over the analytic figure would then be about 2.87 traversals ≈ **the three replay unloads**. That is reading R2, with an unload method more expensive than measurement-based unlookup.
  - V3 must confirm this from the variant's code and a measured run, and must also price the A = O predicate, which the harness never exercises (V1 C6).

### Incidental findings (outside this claim; reported because they affect any T comparison)

1. **Census-based dead-gate stripping.** The incumbent removes 9,678 CCX/CCZ and downgrades 2,243 to CX/CZ (build log, `qcb_audit/results/v2_60d61859_official_eval.txt`). The key table (inc `deep_strip_keys.rs:1-18`) was mined as gates that "never fire" over a census of 121.6 M random input pairs. That is empirical, not a proof, so it is a source of error rate rather than anything to do with the addend interface.
2. **The test set is ground.** The incumbent forces `DIALOG_TAIL_NONCE=9000624727621` (inc `mod.rs:2011`). Its own notes report, for the shipped design point, an expected **9.67 classical mismatches and 8.08 phase-garbage batches per 9,024-shot run** (inc `memory/05-qubit-reduction.md:97-111`). Earlier, 0 of 700 random nonces passed with zero failures (inc `memory/02-lambda.md:11-21`). So the incumbent is an approximate circuit that passes only on a test set found by search. **Not measured here; quoted from contestant notes.** This is the subject of teammate claim 1, and it means every T compared in V3/V4 is "T of an approximate circuit on its chosen test set". The instrument can measure the failure rate on fresh test sets by rewriting only the 96 tail X targets, which leaves the circuit's function unchanged (V1 e). Not done; outside V2.

### Process notes

- **Unauthorized download (stopped).** While checking for a toolchain, I ran `cargo --version` from the repo root. The `rust-toolchain` file (1.93.0) triggered rustup's auto-install, which **downloaded about 131 MB from the Rust distribution server** before I stopped it. That broke brief rule 4. A partial `1.93.0` toolchain (177 MB) and the downloads remain under `~/.rustup`, and you chose to keep them.
- **Approved network and execution.** After that, with your explicit approval, the only network access was `cargo fetch --locked` from crates.io (192 crates, 42 s). All builds used the installed stable toolchain with `RUSTUP_TOOLCHAIN=stable`, offline. Running contestant `build_circuit` unsandboxed was also approved.
- **New files.** `qcb_audit/taint_count/` (Cargo.toml, Cargo.lock copied from the repo, src/main.rs) and `qcb_audit/results/*.txt`. Build outputs and extracted sources stay in the session scratchpad. `git status` shows no tracked changes.

### INFERRED items (V2)

1. Mapping each load/unload run to its `set_phase` (by order and Toffoli volume; the sums reconcile).
2. Redesign headroom for a classical-addend adder: ≤ about 325 T/shot (from contestant notes on the n − 1 floor), order 10²–10³ T overall.
3. Windowed costs: one traversal ≈ 2¹⁶ T, measurement-based unlookup ≈ 2⁸ T, 3·x_A and x_A + 1 as free table payloads, `coord_add3x_orig` ≈ +0.6k–1k T.
4. The 3-stage × (lookup + replay) ≈ 393,216 T reconstruction of the paper's +384,708.
5. The incumbent's error rate and nonce grinding (contestant notes, not measured).

### Open issues for V3

- **Find the windowed variant (paper Supplemental Note F).** It is not on the incumbent's build path; `grep` for `qrom|lookup|unlookup|window` across branches is the next step. Then measure it with the same instrument. A QROM's address-controlled Toffolis will show as unconditioned, so V3 needs a per-window split like the one above, plus an address-taint pass.
- **Decide how to treat the error rate.** If the variant and the incumbent sit at different error rates (λ), their T difference is not purely interface cost.

---

## V3: Decomposition of the windowed overhead

### Headline

1. **The paper's windowed-compatible variant is not in this repository.** The brief says the paper places the code in this repo; I could not find it anywhere. Search scope (details below): all 1,120 local branch heads, every blob reachable in the full local history, and all 803 PR heads on `origin`, including 10 PR heads fetched this session. So **V3 could not be measured as specified.** This conflicts with brief §1, and the code wins. The variant's Q = 1,162, T = 1,684,161 and ΔT = +384,708 remain **paper numbers, unverified here**.
2. **Decomposition from the best available evidence.** I used three inputs: (a) the per-traversal QROM cost, **measured this session** from a validated construction already in the repo; (b) the incumbent's offset exposure, **measured in V2**; and (c) the paper's own description of the variant. Reading the paper's description literally — 3 lookup stages, each payload cleared by exact reversible replay — gives **6 × 65,534 = 393,204 T, within 2.2% of +384,708.** *Correction (follow-up Q2):* the per-traversal unit (MBUC spine, 65,534) is not stated in the paper; I chose it because the reversible-spine alternatives overshoot ΔT. So this agreement is a consistency check of a model picked by fit, **not independent evidence**, and the model overshoots by 8,496 T, which I cannot explain. The R1 bound in item 3 does not come from this model or its residual.
3. **Verdict: R2 (lookup overhead).** The specialization discount (R1) is **at most about 0.75k T: ≤ 0.2% of +384,708, ≤ 0.4% of the 188,100 excess.** *Updated after P2; originally quoted as ≈ 1.3k T, ≤ 0.34%.* The bound rests on V2's measurement that the incumbent has 0 offset-conditioned Toffolis and only 1,600 T of A-touching arithmetic, which already runs on qubits. The figure itself is 433 T **measured** (P2: computing 3·x_A in-circuit) plus ≤ 0.32k INFERRED (x_A + 1). Derivation in follow-up Q1 and P2.
4. **🚩 K3 HOLDS.** Part 4, the residual specialization delta, is about 0. The claim must pivot from "measured hidden discount" to "**interface-mismatch risk**":
   - The leaderboard lets circuits load A for free and schedule it off-peak.
   - A faithful windowed port must pay for that, either with lookups (≈ +3 traversals beyond the analytic 3·2¹⁶, if unloads are replayed) or with width (+256…512 qubits to keep A resident).
   - The standard "+16 qubits, +3·2¹⁶" accounting covers only part of that bill for leaderboard-tuned circuits.

### Search for the variant

| Scope | How | Result |
|---|---|---|
| All 1,120 local branch heads (main, 4 named branches, 1,117 submission branches; refs identical to `origin`, see `git ls-remote`) | Indexed 10,523 unique blobs; regex scan with `blob_grep.py` (scratchpad) | No `unlookup`, no 1684161/384708, no "Supplemental"/"Note F". The `qrom` hits are an old GCD binary-trie QROM test (e.g. inc `mod.rs:4626-4798`). `window_add` hits are squarer arithmetic windows |
| Full local history: every reachable blob (`git rev-list --all --objects`, 14,152 unique blobs) | Same scanner, wider patterns (`reversible replay`, `nonzero address/predicate`, `windowed variant/addition`, `routing ancilla`, `3-stage`) | No windowed point-add variant. Only literature notes and planning text |
| `origin` refs (`git ls-remote`: 1 main + 4 named + 1,117 submissions + 809 `refs/pull/*`) | 793 of 803 PR heads already local. The other 10 fetched with `git fetch --no-write-fetch-head` (no local refs created); 247 PR-only blobs scanned | **Found: an analysis layer on PR #15** (`refs/pull/15/head` = `f76fefed`, 2026-07-03, from fork `CaptainEmpower/ecdsafail-challenge`). It studies exactly this question on an older circuit (below). Still no variant of 8e9c9a2 |
| The audited commits themselves (inc, pp, HEAD) | `git grep` for qrom/lookup/address-register/quantum-addend | Nothing beyond the old trie QROM test |

The PR #15 analysis layer is not merged into `main`: 94 commits beyond merge-base `d44cad38`. It modifies the harness (`src/sim.rs`, `src/circuit.rs`, `src/bin/eval_circuit.rs`), so its scored runs are not on this harness. It also predates the incumbent (2026-07-03 vs 2026-07-26), so it **cannot** be the paper's variant of 8e9c9a2. What it does give:

- **ADR 0010 + `analysis/verify/ladder_lookup_cost.py`.** A unary-iteration QROM (Gidney 2018 §III.C), validated exhaustively for w ≤ 6 in both uncompute forms, including a check that removing the MBUC phase fix-ups makes it fail. It counts Toffolis at w = 16. **Re-run this session** (pure Python, stdlib only, isolated scratch dir); output in `qcb_audit/results/v3_pr15_qrom_cost.txt`:
  - reversible read: **131,068 T**;
  - MBUC read: **65,534 T**;
  - **16 ancilla** in both forms.
- **ADR 0012** (`src/point_add/constprop_gap.rs`). The classical-vs-quantum-addend **Toffoli** gap on a 2026-07-02 head was "770 Toffoli (0.05%) of the 1,446,685 static PA", the only A-value-dependent optimization being constant propagation. Not re-run. It agrees with V2.
- **ADR 0013** (`addend_width.rs`). The **width** gap of holding A resident is +256 (one coordinate) to +512 (both) qubits. The reason: the scored peak is tight, and the classical addend is resident only off-peak (coordinate steps at 1,026 qubits vs peak 1,152). Not re-run.
- **ADR 0014** (`qaddend_testbed.rs`). A reduced-width, QROM-fed quantum-addend modular add, simulation-verified. Its "unread" is "a second application of the reversible read", the same unload method the paper attributes to its variant. It is not a full point add.
- **ADR 0015** (`offset_window_encoding.py`). An offset window encoding that structurally removes the A = O table row. Relevant to the critique pass; not re-run.

### Inputs to the decomposition

| Quantity | Value | Status |
|---|---|---|
| Variant Q / T; ΔT vs incumbent | 1,162 / 1,684,161; **+384,708** | paper only; code not found |
| Incumbent T, Q | 1,299,452.699, 1,151 | measured (V2, official scorer) |
| One QROM traversal, w = 16, MBUC spine | **65,534 T** (2¹⁶ − 2), 16 ancilla | measured this session (validated construction) |
| One QROM traversal, reversible spine | **131,068 T** (2¹⁷ − 4) | measured this session |
| Incumbent offset-conditioned Toffolis | **0** | measured (V2) |
| Incumbent A-touching arithmetic | **1,600.1 T** (five coordinate adders, already on qubits) | measured (V2) |
| Incumbent A-load sites | 5, in order x_sub, y_sub, [inverse], add3x, [square, multiply], y_sub_final, rsub_final; sites 1–2 and 4–5 are back to back with 0 T between | measured (V2) |
| "Standard analytic accounting" 3·2¹⁶ | = 3 lookups per windowed addition, each one traversal with a cheap unlookup. Google 2026 per inc-era notes: "three table lookups per windowed point addition" (`src/point_add/lit_tricks.md:181-184` at `64177289`, 2026-04-29, on `main`; deleted from `main` later). Equivalently, one reversible read plus one MBUC unread = 3·2^w − 6 (ADR 0010) | literature, as recorded in repo notes |

### Decomposition of +384,708 (model M1 = the paper's own description, with measured unit costs)

| Part | What | Model | Toffolis | Basis |
|---|---|---|---|---|
| 1 | QROM traversals for loading | 3 stages × 1 MBUC traversal. The natural stages from V2's layout are {x_A, y_A}, {3·x_A mod p}, {y_A, x_A + 1}. A traversal's T does not depend on payload width, so 3·x_A and x_A + 1 can be table payloads | **196,602** | stage count: paper §5.2.3 (unverified), consistent with V2. Unit cost: measured |
| 2 | Unloads | "Exact reversible replay" = re-running each stage's read: 3 more traversals. (Alternative: a measurement-based unlookup, about O(√2¹⁶) T per stage, INFERRED from the literature, would put part 2 at about 1–2k T) | **196,602** | method: paper wording (unverified). Unit cost: measured |
| 3 | A = O conditioning (nonzero-address predicate) | Predicate = OR of 16 address bits: 15 compute Toffolis per evaluation with MBUC uncompute (INFERRED). Conditioning the five A-touching coordinate adders would at most about double their 1,600 T (controlled add ≈ 2n vs n − 1, per inc `memory/03-proven-floors.md:32-38`). What the variant actually conditions is unknown | **≈ 0.05k … 1.7k** (INFERRED) | inferred; the harness never exercises A = O (V1 C6) |
| 4 | Residual specialization (lost classical tricks) | V2: 0 offset-conditioned Toffolis. 5 first-carry Toffolis could have been free `CX if bit` but are already paid. 3·x_A and x_A + 1 cost 0 T as table payloads. If computed from quantum x_A: 3·x_A costs **+433 T, measured** (P2, `coord_add3x_orig`, inc `ec_add.rs:336-351`), and x_A + 1 ≤ 0.32k INFERRED. Constant propagation is sound with respect to A | **0 … ≈ 0.75k** (was ≈ 1.3k before P2) | measured (V2, P2) + INFERRED |
| | **Model total, parts 1 + 2** | | **393,204** | |
| | **Reported ΔT** | | **384,708** | paper |
| | **Unexplained (parts 3 + 4 + model error)** | | **−8,496 (−2.2%)** | |

- **Tolerance.** M1 reconciles to ΔT within **±2.2%**. Parts 3 and 4 are each ≥ 0, so the −8.5k overshoot means the variant is somewhat cheaper than six full MBUC traversals somewhere. Possibilities: one unload is partly measurement-based ("removes routing ancillas by measurement-based uncomputation", paper), a stage is cheaper than a full traversal, or the variant saves arithmetic elsewhere. Resolving this needs the variant's code.
- **Alternatives tested against ΔT:**

| Model | Toffolis | Outcome |
|---|---|---|
| Literature pricing: 3 MBUC lookups, cheap unlookups | ≈ 197k | Explains only 51%; the shortfall is the brief's +188,100 "excess" |
| 3 reversible-spine reads + 3 MBUC unreads | 589,806 | Exceeds ΔT by 53%; **excluded** |
| 6 reversible-spine traversals | 786,408 | **Excluded** |
| **R1-dominant**: parts 1 + 2 at literature pricing, part 4 ≈ +188k | — | **Excluded by measurement.** V2 and P2 cap everything addend-specialized in the incumbent's arithmetic at ≈ 0.75k T, and only 1,600 T of arithmetic touches A at all |

- **What the +188,100 excess is.** 188,106 = 2.87 MBUC traversals, within 4.3% of three replay unloads (196,602). So the excess over the analytic accounting is **unload cost, not a hidden discount**. It follows from two variant design choices: re-looking A up per stage instead of holding it, and clearing payloads by replay instead of measurement-based unlookup.

### Verdict: R2

| Share of +384,708 | Toffolis | % | Reading |
|---|---|---|---|
| Lookups (loads) | ≈ 196.6k | 51% | R2, priced by the analytic 3·2¹⁶ |
| Replay unloads | ≈ 188–197k | 49–51% | R2: the "excess" (design choice; MBUC unlookup would make it ≈ 1–2k) |
| A = O conditioning (part 3) | ≤ ≈ 1.7k | ≤ 0.4% | R2 (interface), INFERRED |
| Specialization lost (part 4, R1) | ≤ ≈ 0.75k (was ≤ 1.3k before P2) | **≤ 0.2%** | R1, bounded by measurement (V2, P2) + ≤ 0.32k INFERRED |
| Model error | −8.5k | −2.2% | needs variant code |

**R1 is negligible. The evidence supports R2.** The model is only as good as the paper's description of the variant: stage count and unload method are unverified, and the code is absent. The *bound* on R1 does not depend on that description; it comes from V2's measurement of the incumbent.

### Kill / pivot flags

- **🚩 K3 HOLDS.** The specialization discount is negligible for the incumbent: 0 offset-conditioned Toffolis (also measured on 3616dbf2 and on the lowest-Q promoted 3493c837), ≤ ≈ 0.75k T of addend-specialized arithmetic (433 T of it measured), against a +384,708 windowed overhead. Per brief §5 the claim pivots to **interface-mismatch risk**: the leaderboard prices loading A at zero and lets the addend sit off-peak. A windowed port must buy that back with lookups (Toffolis) or residency (qubits). The analytic "+16 qubits, +3·2¹⁶" covers this only if the PA can host a resident addend for free, which a tight-peak leaderboard circuit cannot (ADR 0013 measured +256…512 qubits on an older head).
- **K1, K2:** not triggered (V1).
- **New, for the claim's framing:** the brief's premise that the variant's code is in this repo is false as far as this repository shows. Any numeric claim about the variant should cite the paper, not this repo.

### Rough S comparison (INFERRED, for V4 context only)

| Option | Q | T | S = Q × T | vs incumbent |
|---|---|---|---|---|
| Incumbent (leaderboard interface) | 1,151 | 1,299,453 | 1.496 × 10⁹ | — |
| Paper's variant (reload per stage, replay unload) | 1,162 | 1,684,161 | 1.957 × 10⁹ | ×1.31 |
| Hold-A port, rough: one MBUC lookup of (x_A, y_A), 3·x_A and x_A + 1 computed in-circuit, cheap unlookup | ≈ 1,679 (1,151 + 512 + 16) | ≈ 1,366k | ≈ 2.29 × 10⁹ | ×1.53 |
| Reload + MBUC unlookup instead of replay (not built anywhere) | ≈ 1,162 | ≈ 1,299k + 197k + small ≈ 1.50M | ≈ 1.74 × 10⁹ | ×1.16 |

### Open issues (V3)

- **Get the variant's code** from the paper's authors or artifact (Supplemental Note F). Then:
  - run `qcb_taint_count` on it, adding an address-register taint pass so that traversal, replay and predicate Toffolis are counted directly;
  - replace the M1 model with measured parts 1–4;
  - check its error rate (λ) against the incumbent's.
- **The variant's +11 qubits is below the 16-qubit window register** that a QROM-fed call needs live at the peak. Either the peak moved, or the address register is accounted outside PA. Unverifiable without the code.
- **PR #15's ADR 0012/0013 numbers** (770 T; +256…512 q) were not re-run here. Their harness modifications would need diffing first.

---

## Review follow-up (Q1–Q4)

*As first written, this follow-up ran no contestant code: the instrument was re-run (static pass only) on the two `ops.bin` files already built in V2. You then approved build proposals P1 and P2 (end of Q4); their measured results are marked "P1"/"P2" where they replaced estimates.*

### Q1. Where the R1 bound comes from (quoted as ≤ 1.3k; ≈ 0.75k after P2)

**It is neither V2's direct count nor the V3 residual.** It has a measured floor and an INFERRED top:

| Component | Value | Source | Status |
|---|---|---|---|
| Executed Toffolis/shot under an offset-derived condition | **0.000** (inc), **0.000** (pp) | V2 dynamic pass | measured |
| CCX/CCZ emitted under an offset-derived condition | 0 of 1,358,316 (inc); 0 of 1,009,077 (pp) | V2 static pass | measured |
| All op kinds ever executed under an offset-derived condition | X 2,560; BitInvert 5,683; BitStore1 2,877; Push/Pop 14,324 each; **nothing else** | `qcb_audit/results/followup_offset_by_kind.txt` | measured |
| Toffolis inside the five A-loaded windows | 1,600.1 (inc), 1,600.0 (pp). Already paid in full on qubits | V2 | measured |
| Toffolis with a classical-known A control (convertible to `CX if bit`) | 5/shot. Already paid | V2 | measured |
| Replacing free 3·x_A mod p with in-circuit computation from quantum x_A | **+433.4 T measured (P2).** `coord_add3x_orig` = 755.0 T vs the shipped `coord_add3x` = 321.5 T, same 9,024 inputs, both exact and clean. *Before P2 I had estimated ≈ 0.6–1.0k; the doublings are cheap because `mod_double` only needs a 53-bit fold window* | `qcb_audit/results/p2_add3x_vs_orig.txt` | **measured** |
| Replacing free x_A + 1 | ≈ 0.26–0.32k: a quantum increment, or the non-fused path's `mod_neg` (≈ n) | — | INFERRED |
| **Top of the bound** | **≈ 0.70–0.75k** (was quoted as ≈ 1.3k before P2) | sum of the two rows above | 58% measured, the rest INFERRED |
| Alternative if the table stores 3·x_A mod p and x_A + 1 as payloads | 0 extra Toffolis: a traversal's Toffoli count does not depend on payload width | ADR 0010 construction | INFERRED |

- **What is measured:** zero offset-conditioned arithmetic, and the size (1,600 T) of the arithmetic that touches A.
- **What is estimated:** originally, the whole 1.3k was an estimate of two replacement computations. After P2, the 3·x_A part is measured (433 T); only x_A + 1 (≤ 0.32k) remains INFERRED.
- **What it does not use:** 384,708, or anything from the V3 model.
- **What it assumes:** the variant keeps the incumbent's arithmetic. That is unverified, because the variant's code is absent.
- **P2 calibration:** the standalone shipped `coord_add3x` costs 321.5 T. The same phase inside the full incumbent costs 319.8 T (V2 window 4/5). So the standalone test reproduces in-circuit cost to within 0.5%. P2 mirrors 11 shipped TLM settings; it omits the call-index-keyed dead-carry skip.

### Q2. The V3 model's parameters, and the ≈ 8.5k gap

| Parameter | Value used | Origin | Chosen to fit? |
|---|---|---|---|
| Toffolis per traversal | 65,534 (= 2¹⁶ − 2, MBUC spine) | Measured this session by counting CCX in PR #15's unary-iteration QROM at w = 16. The construction is validated by exhaustive simulation only for w ≤ 6; at w = 16 it is counted, not simulated. The reversible-spine form measures 131,068 | **Yes.** The paper (as summarized in the brief) does not say which spine uncompute is used. I picked MBUC because both reversible-spine models (589,806 and 786,408) overshoot ΔT |
| Number of traversals | 6 = 3 loads + 3 unloads | "3 stages" and "exact reversible replay" (one full traversal per unload) from the paper via the brief. V2's site layout is consistent with 3 stages, but I noted that after the fact | Not fitted, but unverified |
| A = O conditioning (part 3) | **0 in the sum**; separately 0.05–1.7k INFERRED (from a 15-Toffoli OR predicate per stage, up to making the five coordinate adders controlled) | inferred | — |
| Residual specialization (part 4) | **0 in the sum**; 0–0.75k separately (Q1 + P2; was 0–1.3k) | V2 + P2 + INFERRED | — |
| **Model total vs ΔT** | 6 × 65,534 = **393,204** vs 384,708: overshoot **8,496 (2.21%)**. With 2¹⁶ per traversal (your 393,216): 8,508. Adding parts 3–4 widens it to ≈ 8.5–11.0k (2.2–2.9%) | | |

- **The 8.5k cannot be explained from evidence here.** Untested candidates:
  1. The variant's lookups cost less than 2¹⁶ per traversal: fewer live table rows, or a different unary-iteration layout.
  2. Part of one unload is measurement-based. The paper mentions measurement-based uncomputation for routing ancillas.
  3. The variant's arithmetic is ≥ 8.5k cheaper than the incumbent's, from a different tuning, census-strip table or nonce.
- **Shot noise is not the cause.** The HMR-conditioned share (117,747 gates firing on 50% of shots) has a standard error of about 1.8 T/shot over 9,024 shots.
- **Conclusion for Q2: the "±2.2% reconciliation" carries no evidential weight on R1 vs R2.** It checks consistency of a model selected from a small discrete set. The robust facts are:
  - ΔT = **5.87 MBUC-traversal equivalents** (paper ΔT ÷ measured unit);
  - V2's measured absence of offset-conditioned arithmetic. The R1 bound rests on this alone.
- I corrected V3's headline accordingly (item 2).

### Q3. Every place the offset bits enter 8e9c9a2

Source references are inc `trailmix_ludicrous/ec_add.rs`; window columns are measured.

| # | Phase (line) | Addend | Operation | How the offset bits are used | Window in op stream / T |
|---|---|---|---|---|---|
| 1 | `tlm_coord_x_sub` (288) | x_A, raw | x ← x − x_A mod p (`mod_sub_vented`, 44) | 256 conditioned X load + unload (38-41, 48-50) | runs 0/1 · 320.1 |
| 2 | `tlm_coord_y_sub` (290) | y_A, raw | y ← y − y_A mod p | same | runs 2/3 · 320.0 |
| 3 | `tlm_coord_add3x` (297) | **3·x_A mod p**, derived classically (84-154) | x ← x + 3·x_A mod p (`mod_add`, 68) | classical bit arithmetic on `ox`, then load + unload (62-74) | runs 4/5 · 319.8 |
| 4 | `tlm_coord_y_sub_final` (307) | y_A, raw | y ← y − y_A mod p | load + unload | runs 6/7 · 320.2 |
| 5 | `tlm_coord_rsub_final` (309) | **x_A + 1 mod 2²⁵⁶**, derived classically (217-229) | x ← x_A − x mod p (`mod_rsub_vented_loaded`, 241) | classical increment, then load + unload (236-250) | runs 8/9 · 320.0 |

**Answer: all five are modular additions or subtractions.** Three use the raw coordinates (x_A once, y_A twice). Two use classically derived constants (3·x_A mod p; x_A + 1). A never enters the inversion, square or multiply directly; those act only on quantum data derived from x − x_A and y − y_A.

The op stream confirms it: the only quantum op kind ever executed under an offset-derived condition is **X (2,560 = 10 × 256)**. The rest is classical bookkeeping: BitInvert 5,683, BitStore1 2,877, Push/Pop 14,324 each. No CX, CZ, Swap, CCX, CCZ, R, HMR or Neg is ever offset-conditioned. 3616dbf2 gives identical counts.

### Q4. Other circuits

#### The three lowest-Q promoted submissions (landed on `main`), from metadata only

**Method.** PR bodies and bot comments are not in git. Per your list of sources, I read them through GitHub's REST API: read-only, unauthenticated.
- 9 page requests covered all 803 PRs.
- Bot comments were fetched for 51 PRs, until the 60-per-hour limit stopped the rest. About 85 requests in total, including 25 that failed on the limit.
- **Q/T/S below come from the CI bot's "Scored" comment** (`yukon-autoresearch[bot]`) on each Validate PR. Submitter body claims were used only to shortlist; they can quote baselines first (PR #56's body names 1,150, but its bot score is Q 1,321).
- Evidence: `qcb_audit/results/followup_lowest_q_promoted.txt`.

| Rank | Submission | Landed on `main` | PR | Q | T (bot, rounded) | S |
|---|---|---|---|---|---|---|
| 1 | `3493c837-5e3f-403c-bae7-1cd5467ab719` | `31f9c58c`, 2026-08-20 | #53 | **1,150** | 1,284,776 | 1,477,492,400 |
| 2 | `b87bdf71-f9d4-43b2-b31c-6b9652b7946c` | `04224373`, 2026-08-19 | #52 | 1,153 | 1,281,867 | 1,477,992,651 |
| 3 | `cb4540c2-f6cf-4dd8-89d0-408748e399ce` | `afcc72ef`, 2026-08-18 | #51 | 1,153 | 1,284,345 | 1,480,849,785 |

**Limits of this ranking:**
- **It covers only the 207 submissions that have PRs**, i.e. the post-August Validate merges. The 428 earlier "Accept submission" commits carry no metrics anywhere:
  - no PR;
  - no `score.json` in any blob of history;
  - `results.tsv` has 1,259 OK rows, almost all ~2,700-qubit development runs; its one 1,152-qubit row matches no submission.

  So a lower-Q Accept submission cannot be ruled out from metadata. The incumbent (an Accept, Q 1,151 measured in V2) would rank 2nd if included.
- **25 candidate PRs were not fetched** because of the rate limit. Their titles claim Q 1,170–1,174 (the Skywalk lineage) or 1,236–1,237; #661 states no Q.
- **Cross-check:** PR #56's bot result for 3616dbf2 (Q 1,321, T 952,707, S 1,258,525,947) equals this audit's V2 measurement exactly.

#### V2(a–b) for each circuit

| Circuit | Offset-conditioned CCX/CCZ | Offset-dependent executed T/shot | T in the A-loaded windows | Addend-access windows | Basis |
|---|---|---|---|---|---|
| Incumbent 8e9c9a22 (`60d61859`) | 0 / 1,358,316 | **0.000** | 1,600.1 | 5 | measured |
| 3616dbf2 (`897dda2b`) | 0 / 1,009,077 | **0.000** | 1,600.0 | 5 | measured |
| **3493c837 (`31f9c58c`), lowest-Q promoted** | **0 / 1,340,596** | **0.000** | **1,592.6** (318.0 / 317.9 / 318.9 / 319.9 / 317.9) | **5** | **measured (P1)**, consistent with the source reading below |

**P1 results (3493c837 at `31f9c58c`).** Source in `qcb_audit/results/p1_3493c837_31f9c58c_official_eval.txt` and `p1_3493c837_31f9c58c_taint.txt`.
- **Official scorer:** **Q 1,150, T 1,284,776.069** → score **1,477,492,400**, exactly the CI bot's numbers. All 9,024 shots OK.
- **Op stream:** 9,018,685 ops; 518,531 classical bits.
- **Run cost:** `build_circuit` 21 s at a 1.44 GB peak; eval 25 s at 0.49 GB; instrument 37 s at 0.59 GB.
- **Instrument:** identical in structure to the incumbent.
  - 0 offset-conditioned CCX/CCZ. Of the 1,340,596 CCX/CCZ, 111,665 are HMR-conditioned and execute 55,845.1 T (50.0%).
  - The offset-conditioned op kinds are the same as the incumbent's: X 2,560, BitInvert 5,683, BitStore1 2,877, Push/Pop 14,324 each.
  - The same 5 first-carry Toffolis have a classical-known A control.
  - Windows reconcile: 1,592.6 + 611,638.9 (division) + 671,544.6 (square + multiply) = 1,284,776.1 = T.
- The build log shows a post-pass the incumbent lacks: `[q1150-source-rebound] applied 16850 pinned actions`. It did not move any gate into an offset-conditioned block.

**Source evidence for 3493c837 (read before P1, now confirmed):**
- **Harness and route.** The harness is identical to HEAD, and the build takes the TrailMix route (`31f9c58c:src/point_add/mod.rs:2520`).
- **Offset code.** `ec_add.rs` is semantically identical to the incumbent's (empty diff ignoring whitespace/CR), with the same flags `TLM_COORD_RSUB_FUSED=1` and `TLM_COORD_ADD3X_TRUNC=1` (`mod.rs:1197, 1199`). The offsets are allocated and passed only to `ec_add` (`trailmix_ludicrous/mod.rs:460-465`).
- **Call sites.** Per-file counts of `x_if_bit`/`x_if`/`z_if_bit` call sites match the incumbent's.
- **Condition sources.** The conditioned-emit source lines (`push_condition`, `c_condition =`) are textually identical: 13 = 13. The only difference is 4 `.orig` backup files the incumbent ships and `31f9c58c` removed.
- **Result:** the same five addend-access windows (10 load/unload runs), the same two derived constants, and every other condition source is an HMR outcome.
- **Not established without a build:** whether post-passes reorder any gate into an offset-conditioned block. In both measured circuits they did not. *P1 settles it for 3493c837: they did not.*

#### Build proposals — both approved and run after the follow-up

| Proposal | Measures | Ops (estimate → actual) | Peak RAM (estimate → actual) | Runtime (estimate → actual) | Runs contestant code? |
|---|---|---|---|---|---|
| **P1:** build + official eval + instrument for 3493c837 (`31f9c58c`) | measured V2(a–b), including per-window T | ≈ 9.1 M → **9,018,685** | ≈ 1.5 GB → **1.44 GB** (build) | ≈ 4 min → about 3 min (compile ≈ 1.5 min, build 21 s, eval 25 s, instrument 37 s) | yes: its `build_circuit`, unsandboxed, in a scratch extraction |
| **P2:** add3x vs add3x_orig micro-benchmark on the incumbent | the 3·x_A part of the Q1 bound, measured | ≈ 10⁴–10⁵ → **41,995** (shipped) / **12,409** (orig) | < 0.2 GB → negligible | ≈ 2 min → 27 s compile + 1 s run | yes: calls the incumbent's public `build_add3x_test` / `build_add3x_test_orig` (`mod.rs:23` → `trailmix_ludicrous/mod.rs:6` → `ec_add.rs:326, 353`) from a new bin in the scratch copy, without editing contestant files |

---

## Critique pass

The claim under test is QCB's: "the leaderboard rewards classical-addend optimizations that do not transfer to a real (windowed) attack." Below are the brief's four objections plus yours (C2).

### C1 — "The overhead is mostly lookup cost, already priced by the analytic formula" (reading R2). **Status: FATAL to the hidden-discount (R1) headline.**

- **For the objection:**
  - V2 measured 0 offset-conditioned Toffolis. The addend's only free advantages are its loads (2,560 conditioned X) and two precomputed constants.
  - ΔT = 5.87 traversal-equivalents. Nothing in the incumbent's 1,600 T of A-touching arithmetic could supply 188k of specialization.
  - The interface is declared, not hidden: `benchmark.json` describes the task as adding "a quantum elliptic curve point to a classical elliptic curve point", and the README says the same.
- **Partial rebuttal:** the analytic accounting (+16 qubits, +3·2¹⁶ T) under-prices *this kind* of PA:
  - ADR 0013 (PR #15, older head, not re-run) measured +256…512 qubits to hold A across a tight peak.
  - The paper's variant instead pays about +188k T beyond the analytic figure (paper number).
  - So the leaderboard's low Q is partly achieved *because* A is free and off-peak.
- **What survives** is a weaker claim: "analytic lookup accounting under-prices leaderboard-tuned PAs, by up to about 2× on the lookup term in T, or +22–44% in Q." Stating it precisely needs the variant's code.

### C2 — "The 1.3k bound is circular." **Status: PARTIALLY RESOLVED.**

- **Not numerically circular.** The bound is built from V2 measurements (0 offset-conditioned T; 1,600 T A-touching arithmetic, already on qubits; 320 T per adder) plus INFERRED replacement costs. It uses neither 384,708 nor the V3 model (Q1). V3's former wording implied otherwise; that is now corrected.
- **Two real dependencies remain:**
  1. **Taxonomy.** Replacing the free conditioned-X loads (hypothesis (c)) is booked as R2 "lookup overhead", following the brief's R2 definition. Count (c) instead as "classical conditioning the leaderboard never pays for", and R1 becomes about the entire lookup bill: the same numbers, relabelled. So R1 ≈ 0 means "**no arithmetic-level discount beyond loading A**". It does not mean "no interface advantage".
  2. **Assumption.** The bound assumes the variant keeps the incumbent's arithmetic, which cannot be checked without its code.
- **Mostly measured now.** The 1.3k was INFERRED when this critique was written. After P2 the bound is ≈ 0.75k: 433 T measured (3·x_A replacement), plus ≤ 0.32k INFERRED (x_A + 1). P1 also measured 0 offset-conditioned Toffolis on the lowest-Q promoted circuit.
- **Still open:** obtain the variant's code (dependency 2), and state the taxonomy explicitly in the claim (dependency 1). The status stays **PARTIALLY RESOLVED**, because both remaining dependencies are about scope, not measurement.

### C3 — "Double-and-add uses a classical addend legitimately." **Status: PARTIALLY RESOLVED.**

- **What the objection gets right.** In double-and-add the addend 2ⁱ·P is classical at compile time, so (b)/(c)-style specialization is legitimate there, even at build time.
- **But every addition must be controlled by an exponent qubit.**
  - The harness has no control register: exactly 4 registers (`eval_circuit.rs:471-508`).
  - "Control = 0 ⇒ identity" is the same exceptional-case problem as A = O, which the harness never tests (V1 C6).
  - With the incumbent's structure, a controlled *load* would still be Clifford (CX from the control into the temp register). The extra cost would sit in the identity handling (INFERRED).
- **Call counts.** About 2n = 512 controlled additions versus 2n/w − 4 = 28 windowed ones (paper formula as recorded in the brief and PR #15 ADR 0010). At leaderboard T that is about 512 × 1.30M ≈ 665M T versus about 28 × (1.30M + 6 × 65,534) ≈ 47M T (INFERRED). The leaderboard PA's realistic use is therefore the windowed ladder, where A is quantum.
- **Net.** The interface matches neither attack mode exactly: windowed needs quantum A, double-and-add needs a control qubit. The objection does not make the leaderboard representative, but it does show classical-addend tricks are not illegitimate in themselves.

### C4 — "Random offset masks remove the A = O cost" (arXiv:2609.05625). **Status: RESOLVED (the objection holds).**

- The harness never tests A = O (V1 C6).
- PR #15 ADR 0015 removes the ∞ table row at 0 Toffolis with an offset window encoding (shift every digit by 1 so index 0 is never selected). It was validated on a toy curve in Python; not re-run here.
- The part-3 estimate is ≤ about 1.7k in any case (INFERRED). The A = O term doesn't carry the claim either way.

### C5 — "The V1 mechanism differs from the brief." **Status: RESOLVED.**

- **V1 confirmed the mechanism.** Offsets are classical bits, conditioned Toffolis count only when the condition is true, and free classical bit logic exists. K1 and K2 are not triggered.
- **Minor corrections:**
  - hypothesis (b) is run-time classical computation, not build-time precomputation;
  - Q is the highest qubit index + 1;
  - the README is stale.
- **Nuance from V2/follow-up:** the mechanism exists, but all three measured or source-read circuits use (a) zero times. They use (c) (free X loads) and (b) (classical 3·x_A and x_A + 1) only.

### The three strongest objections

1. **C1** — fatal to the discount headline.
2. **C2** — partially resolved.
3. **C3** — partially resolved.

C4 and C5 are resolved.

### Net position

- **Drop the R1 / "hidden discount" headline (K3).**
- **The supportable pivot is interface-mismatch risk:**
  - The leaderboard declares a classical addend, and its winning circuits reach their low Q partly by loading A for free and freeing it between uses.
  - A windowed port must pay either about +256…512 qubits to hold A, or about +3 traversals beyond the analytic accounting. The paper's variant reports +384,708 T against an analytic 196,608.
- **Missing evidence:** the variant's code.

### Incidental (affects any T comparison, not this claim)

- **Approximate circuits are now explicitly allowed.** Recent PR bodies (#429, #430, #433, #434) cite a "Circuit Lambda … contract limit of 18" from PR 419: up to about 18 expected failure events per 9,024-shot run. Their λ values (about 16.3–16.5, over 512 nonces) are submitter claims, not verified.
- **So leaderboard T values are T of approximate circuits.** This matches the incumbent's own notes (V2 incidental 2).

### Process notes (follow-up)

- **No contestant code was run** for the follow-up itself.
- **Network:** GitHub REST API only, read-only and unauthenticated, about 85 requests (25 refused by the rate limit). This was at your direction, since the requested PR bodies are not stored in git.
- **New files:**
  - `qcb_audit/results/followup_offset_by_kind.txt`;
  - `qcb_audit/results/followup_lowest_q_promoted.txt`;
  - a `--static` flag and a per-kind table in `qcb_audit/taint_count/src/main.rs`.

### Process notes (P1 / P2, run on your approval)

- **P1** ran 3493c837's contestant code (`build_circuit` at `31f9c58c`) unsandboxed, in a `git archive` scratch extraction, under the 8 GB watchdog:
  - release build with stable 1.99.1, `--locked --offline`;
  - then the official `eval_circuit` (its `score.json`/`results.tsv` went to the scratch copy) and the instrument.
- **P2** compiled a new bin, `qcb_audit/add3x_bench/qcb_add3x.rs`, into the incumbent's scratch extraction. That tree's library exposes `point_add`, so no contestant file was edited. The copy's SHA-256 is recorded in `qcb_audit/results/p2_add3x_vs_orig.txt`.
- **No network** was used for either.
- **New files:**
  - `qcb_audit/add3x_bench/qcb_add3x.rs`;
  - `qcb_audit/results/p1_3493c837_31f9c58c_official_eval.txt`;
  - `qcb_audit/results/p1_3493c837_31f9c58c_taint.txt`;
  - `qcb_audit/results/p2_add3x_vs_orig.txt`.
- `git status` shows no tracked changes.

---

## V4: Ranking sensitivity

*Per your instructions, V4 uses source and metadata only. No builds and no contestant code. Claims taken from PR bodies are labeled **(claim)**.*

### Headline

1. **The top-ranked lineage does exploit hypothesis (a), unlike the incumbent.** It does so in two ways:
   - **By construction, in all three top ranks.** Its classical-operand adder computes the first carry as a classically conditioned CX on an offset bit.
   - **Through generic "classical-factor" passes, in ranks 1–2 only.** These rewrite any CCX whose control provably equals a classical bit — including offset bits — into a conditioned CX.

   That said, the second category is not literally V2's 5 first-carry sites: the x-side arithmetic is structured differently (details below).
2. **The top three are separated by 25.0 T and 19.0 T** at a shared Q of 1,234. The passes' claimed saving is 7.5 T **(claim)**. Even if all of it were offset-derived and charged back (Q × 7.5 = 9,255), it is smaller than both S gaps (30,850 and 23,446).
3. **The windowed adjustment** (Q + 16, T + 198,138) **does not change the order.** It is uniform across three circuits with the same Q and the same addend-access structure, so it shifts all three by the same rule.
4. **Ranking verdict:** under any windowed-interface score that treats these three alike, the top-3 order is unchanged. A flip would need rank-specific offset-derived savings of at least 19 T (rank 2 vs 3) or 25 T (rank 1 vs 2). That is 2.5–3.3× the passes' claimed total, and only an op-stream measurement can rule it in or out.

### 1. The classical-factor passes in 54fc4f25 (rank 1) and d9ae2845 (rank 2)

**Same code in both.** `git diff --stat 90e777ea 3dee0cd6` on `classical_factors_xor.rs` and `classical_wide/` is empty. Rank 3 (`0c3a0751`) has neither file and does not call them. Ranks 1–2 call them at `mod.rs:620` and `mod.rs:622`, after the other simplifiers and before the nonce tail.

**What they do** (`classical_factors_xor.rs` at 3dee0cd6; `classical_wide/mod.rs` is the same scheme with up to 9 atoms instead of 7):

| Step | Code | Effect |
|---|---|---|
| Atoms | `Support::new`, lines 98–128 | Every ABI register slot gets a fresh truth-table atom: the target **qubits** *and* the offset **bits** (`classical_inputs`, lines 124–127). Classical bits are then tracked through `BitInvert`/`BitStore*`/`Hmr` |
| Index | `simplify_classical`, line 378 | Maps each non-constant classical bit's truth table to the bit ids holding it |
| Direct match | lines 384–386 | For each CCX: if one control qubit's truth table equals some classical bit's, that bit is the witness |
| Factor search | lines 388–401 | Otherwise: if (condition ∧ product of the controls) = (condition ∧ one control ∧ a classical bit or its complement), the bit is the witness |
| Rewrite | lines 409–421 | `CCX(q, other, t)` → `PushCondition bit; CX(other, t); PopCondition`, plus one extra CX if complemented. A "proxy" saving of 2^−depth is logged |

- **Do they condition gates on offset bits?** **Yes, by construction.** They are value-agnostic. Offset bits are atoms, and any classical bit whose truth table matches a control qualifies, offset-derived or measurement-derived. The passes' own doc comment says "classical bits already contain their values" (lines 373–374). Which witnesses they actually choose on these circuits cannot be read from source; it needs the op stream (not built, per your instruction).
- **Are they V2's 5 first-carry sites?** **Same category, not the same sites.** V2's 5 sites in the incumbent are CCX whose control is a temp qubit holding a pure copy of an offset bit; this pass would rewrite exactly such gates. But ranks 1–3 load A differently:

| Offset window in ranks 1–3 | Code | First carry |
|---|---|---|
| x − ox (seamed into the division's seed) | `leapfrog.rs:3661-3666` → `modular::r5_ripple_add_cbits` | **Offset-conditioned CX by construction** (`modular.rs:1004-1005`: `push_condition(a[0]); cx(acc[0], carries[0])`). Later carries are CCX with offset-conditioned X flips. The top-k compare loads k operand bits into `tv` by conditioned X (`leapfrog.rs:3704-3708`) |
| x + 3·ox + const (division unseed) and x ← ox − x (multiply unseed) | `divide_seamed`/`multiply_seamed` (`leapfrog.rs:6326-6342`) → `seam_add_reduce` (`3782-3830`, adder at 3792) with classical operands from `classical.rs:152` (`add3x_operand`) and `classical.rs:164` (`plus1_operand`) | The same built-in conditioned-CX first carry. Compare windows again load `tv` from operand bits (`leapfrog.rs` around 3818-3822) |
| y − oy twice (`coord_y_sub`, `coord_y_sub_final`) | `classical::coord_sub` (`classical.rs:52-77`). The pinned `R5_CBITS=15` (`mod.rs:431`) leaves bit 16 clear, so it falls back to `against_coord(..., mod_sub_vented)` (`classical.rs:75`): the incumbent-style temp-register adder | A plain CCX on a temp copy of `oy[0]`: **the surviving analogue of V2's first-carry sites**, a candidate for the passes in ranks 1–2 |

- **Route.** Seams are on in all three ranks: `LEAPFROG=1`, `LF_SEED=half`, `LF_SEAMS=1` (`leapfrog.rs:306, 309, 374`; `seams() = lf_seams() && seed_half()`).
- **Five addend-access windows**, as in the incumbent: three x-side (seamed) and two y-side.
- **The three x-side first carries are offset-conditioned CX in all three ranks**, with or without the passes. Relative to a quantum-addend port, they save about 3 T/shot uniformly, so they cannot affect ranks 1–3 relative to one another.
- **What the passes add in ranks 1–2** cannot be pinned to sites without the op stream. Plausible targets (INFERRED) are the two y-side first carries and the compare windows fed by `tv` bits loaded from offset operands.
- **Claimed size:** PR #817 states "the two general passes … run unchanged on the new gate list: −6.5 and −1.0" expected executed Toffolis **(claim)**.
- **One more unknown:** rank 1's "regenerated AND cache" and rank 2's "frozen cache" use "classical flags" **(claim, PR #817/#814)**. Whether those flags are offset-derived is not determinable from the PR text or a quick source read.

### 2. Ranks 1–3: S, Q, T and gaps

`results.tsv` records none of these (V2 follow-up Q4). Every S below comes from the Yukon bot's "Current best score" line in the next promotion's PR body, except rank 1's, which only its own PR body states. Evidence: `qcb_audit/results/v4_top3_metadata.txt`.

| Rank | Submission | Landed on `main` | S | Source of S | Q | T (= S/Q) |
|---|---|---|---|---|---|---|
| 1 | 54fc4f25 | `3dee0cd6` (= HEAD), PR #817 | **939,138,168** | **(claim)** in PR #817's title and body. Promoted (merged), so it beat 939,169,018. The exact bot comment was not fetched (rate limit) | 1,234 **(claim)** | 761,052 (body: 761,052.442 **(claim)**) |
| 2 | d9ae2845 | `90e777ea`, PR #814 | **939,169,018** | bot-recorded: "Current best score" in PR #817 and #818 bodies | 1,234 **(claim)** | 761,077. PR #814's own table repeats rank 3's 761,096.082, which is inconsistent with its S; I use S/Q |
| 3 | 9dc3798c | `0c3a0751`, PR #718 | **939,192,464** | bot-recorded: "Current best score" in PR #814 body | 1,234 **(claim)** | 761,096 (body: 761,096.082 **(claim)**) |

- **Integer check:** each S is an exact multiple of 1,234.
- **Adjacent S gaps:** rank 2 − rank 1 = **30,850** (= 25.0 T at Q 1,234); rank 3 − rank 2 = **23,446** (= 19.0 T).

### 3. Flip bounds

**(a) Charging back the passes' claimed saving.** Assume the full 6.5 + 1.0 = 7.5 T **(claim)** is offset-derived. Each such Toffoli then costs one Toffoli per execution again with a quantum addend (the per-gate model validated in the instrument's positive control). Q × 7.5 = 1,234 × 7.5 = **9,255**.

| Comparison | S gap | Q × 7.5 | Gap minus penalty | Flips? |
|---|---|---|---|---|
| rank 1 vs rank 2 | 30,850 | 9,255 | +21,595 | **No.** Holds even in the worst case where only rank 1 is charged, although both carry the identical passes |
| rank 2 vs rank 3 | 23,446 | 9,255 | +14,191 | **No.** Only ranks 1–2 have the passes, so rank 3 gets no penalty |

**Charged leaderboard S:** rank 1 = 939,147,423; rank 2 = 939,178,273; rank 3 = 939,192,464 (no passes). The order is unchanged.

**(b) Windowed adjustment** (your parameters): Q + 16; T + 3·(2¹⁶ − 2) + 3·512 = T + 196,602 + 1,536 = **T + 198,138**.

| Rank | Q_w | T_w | S_w = Q_w × T_w | S_w with the 7.5 T charge-back (ranks 1–2) |
|---|---|---|---|---|
| 1 | 1,250 | 959,190 | **1,198,987,500** | 1,198,996,875 |
| 2 | 1,250 | 959,215 | **1,199,018,750** | 1,199,028,125 |
| 3 | 1,250 | 959,234 | **1,199,042,500** | 1,199,042,500 (no passes) |

- **Adjacent windowed gaps:** 31,250 and 23,750 (1,250 × the same 25 T and 19 T). **The order does not change** with or without the charge-back.
- **Why the order holds:** with equal Q and the same five-window addend structure (section 1), a uniform additive (ΔQ, ΔT) preserves the T order exactly.
- **What could flip it:** only *rank-specific* windowed costs. That would take ≥ 19.0 T (2 vs 3) or ≥ 25.0 T (1 vs 2) of extra offset-derived savings in the better-ranked circuit. That is 2.5× and 3.3× the passes' claimed total.

### Verdict and limits

- **Ranking:** unchanged under a windowed-interface score for the top three. This rests on PR claims (Q, T, and the 7.5 T pass saving) and on bot-recorded S values; none of it is measured here.
- **Exposure:** the top lineage does use classical-addend conditioning, which the incumbent did not:
  - built-in conditioned-CX first carries on the three x-side windows, in all three ranks;
  - generic classical-factor rewrites in ranks 1–2.

  Its x-side arithmetic also takes the offsets as classical operands without a 256-qubit temp register. A quantum-addend port would need A in qubits there, which may cost width if those steps sit near the peak (INFERRED; uniform across ranks 1–3, so not rank-changing).
- **Across lineages the order is robust by a wide margin.** The lowest-Q promoted circuit (3493c837: Q 1,150, T 1,284,776, S 1.477 × 10⁹) stays far behind the top (0.939 × 10⁹) under the same adjustment: (1,166 × 1,482,914) ≈ 1.729 × 10⁹ vs 1.199 × 10⁹. In the earlier comparison (V2 follow-up), flipping an order between such different (Q, T) points needed ΔT > 6.5 M per call.
- **Open (would need the op streams; builds not run per your instruction):**
  1. the exact sites and sizes of the passes' offset-conditioned rewrites in ranks 1–2;
  2. whether the AND caches' "classical flags" are offset-derived;
  3. whether rank 1's claimed T (761,052.442) and S are bot-confirmed. PR #817's bot comment was not fetched.

  The instrument's per-kind table and windowed-restore estimate would answer 1 and 2 directly; its positive control is described below.

### Process notes (V4)

- **No builds and no contestant code.** Sources were read with `git show`/`git grep` at `3dee0cd6`, `90e777ea` and `0c3a0751`.
- **No new network:** the PR metadata had already been fetched in the follow-up.
- **Instrument extension, tested only on the three op streams already built:**
  - What it adds: the per-op count of OFF-tainted condition factors, executions under the condition with those factors removed, and a modelled per-execution Toffoli cost.
  - On the three built streams it returns 0.000 extra T/shot, consistent with V2.
  - **Positive control** on a synthetic stream (`synth_ops.py`, scratchpad) with offset-conditioned CX and CCX: it reports the expected 4.993 T/shot (2 CX × 1.0 + 2 CCX × ≈ 1.50).
- **New file:** `qcb_audit/results/v4_top3_metadata.txt`.
