<!-- PARTIAL capture, 2026-10-29, from the Discord preview and attachment viewer (Aodhfionn Downs, thread "Error pricing undercharges approximate circuits by 2x in the exponent, and phase errors by 4x", Oct 22). Gaps are marked [not captured]. Download the original (17 KB) to replace this file. -->

# C-AMP: Error pricing is off by 2x in the exponent (4x for phase errors)

Self-contained. No other thread file is needed to follow it.

**Type:** claim (the proposed and published error pricing undercharges approximate circuits) plus a scoring fix. **Status:** derivation, exact toy simulation and leader measurement done. Every number below has a reproduction command in section 7. **Repo state measured:** ecdsafail/ecdsafail-challenge at 72ccdb8 (HEAD on 2026-10-22), built with stable Rust 1.97 (pinned 1.93 was not downloadable here; the evaluator is unchanged).

**Targets:**
- P003 (per-run error pricing): Score = T̄·Q·(1−ε)^(−k);
- Elliot's reply to P003 (k = 28, ε_max ≈ 0.376% for 90% run success);
- joel's Track G synthesis: ((1 − ε_0)/(1 − ε_UB))^K;
- C-MOD F8 (Q·T/(1 − ε_UB)), R002 ("price error into the score"), and the ECDSA.Fail paper (arXiv 2609.09582), which reports Q×T/p̂ per call.

All of these share one model. Each addition fails like an independent classical coin with probability ε, so a run of k additions succeeds with probability (1−ε)^k. That model is wrong for a coherent Shor run.

## 1. Claims

**C1.** In a windowed Shor run, a point adder that is wrong on a fraction ε of uniformly random inputs reduces the overlap amplitude with the ideal state by about ε per addition (2ε for phase errors), and success probability goes as the square of that overlap:

    P_run ≈ P_ideal · (1 − ε_W − ε_A)^(2k) · (1 − 2·ε_P)^(2k)

where ε_W is the fraction of inputs with a wrong output, ε_A the fraction with a correct output but a dirty ancilla, and ε_P the fraction with correct output, clean ancillas and a nonzero phase. To first order, P_run ≈ exp(−2k(ε_W + ε_A + 2ε_P)).

**C2.** The classical model (1−ε)^k overstates P_run. It is optimistic, so it rewards adding error. Relative to the truth it undercharges wrong outputs and dirty ancillas by 2x in the exponent and phase errors by 4x.

**C3.** Exact simulation of windowed discrete-log phase estimation with a faulty adder (section 3) measures loss exponents of 1.5 to 1.9 for wrong/ancilla errors and 3.0 to 3.7 for phase errors where the classical model predicts 1. The amplitude model reproduces the simulation exactly for phase errors, and to within a residual that vanishes as each failing input covers a smaller share of paths (2^-32 or less in real windowed Shor).

**C4.** The current leader (HEAD 72ccdb8, 743,575 avg Toffoli × 1,242 qubits = 923.5M), audited on 128,000 independent inputs, fails 0.200% of inputs with a wrong output and 0.036% with phase only. At k = 28 the classical model prices this at +6.8% (run success 93.6%); the amplitude model at +16.5% (run success 85.9%).

**C5.** The 90% run-success gate proposed in the P003 thread (ε ≤ 0.376% at k = 28) becomes ε_W + ε_A + 2ε_P ≤ 0.188%. The leader is at 0.272% and fails it.

## 2. Derivation

**A.1 Setup.**
- Shor for ECDLP prepares exponent registers e = (a, b) in uniform superposition and computes the accumulator f(e) = aP + bQ with k windowed additions.
- Before the final QFT the ideal state is |Ψ⟩ = Σ_e α_e |e⟩|f(e)⟩|0_anc⟩.
- Group paths e by the value v = f(e); each group is a coset line of the kernel lattice.
- The QFT on e followed by classical post-processing succeeds with probability ≈ 1 when every line is complete and coherent.

**A.2 Faulty adder.**
- Let S be the set of paths on which at least one addition fails, and δ = Σ_{e∈S}|α_e|². With per-addition failure fraction ε on effectively uniform inputs, δ = 1 − (1−ε)^k.
- For wrong outputs and dirty ancillas, the failed branch moves to work-register states orthogonal to every good branch, so ⟨Ψ|Ψ'⟩ = Σ_{e∉S}|α_e|² = 1 − δ.
- For a phase error (sign flip), ⟨Ψ|Ψ'⟩ = 1 − 2δ_odd, where δ_odd is the weight of paths with an odd number of flips.
- Write Ψ' = ⟨Ψ|Ψ'⟩·Ψ + r with r ⊥ Ψ. When the ideal success probability is near 1 and success outcomes are a small fraction of the outcome space, r contributes almost nothing, so:
    - P_run/P_ideal ≈ |⟨Ψ|Ψ'⟩|² = (1 − ε)^(2k) for wrong output or dirty ancilla;
    - P_run/P_ideal ≈ (1 − 2ε)^(2k) for phase.
- The classical model uses 1 − δ, the probability that a path is unaffected, in place of its square.
- The exact formula used in the simulation (group Z_N of prime order) was checked against a brute-force state-vector DFT on N = 11. [formula not captured]

**A.3 Dirty resets.**
- A dirty R leaves an unknown random sign; the run is the mixture ½(Ψ + Ψ_flip), with success ½[1 + (1−2δ)²] ≈ 1 − 2δ.
- The harness flags only the half of such shots where its simulated outcome is 1. Weighting detected phase failures by 2 gives 2·2·(δ/2) = 2δ, so the class weights in C1 stay correct for dirty resets.

**A.4 Relation to the literature.**
- Gidney, "Approximate encoded permutations and piecewise quantum adders" (arXiv 1905.08488), bounds the worst case: overlap ≥ 1 − 2ε per deviation ε, and trace distance ≤ 2√ε. This is consistent with C1 and looser than it; C1 is the typical-case success probability for unstructured failures.
- The ECDSA.Fail paper prices one call (Q×T/p̂) and does not compound over 28 calls. P003 compounds classically.

## 3. Verification by exact simulation

Code: shor_error_toy.py.
- **Model.** Group Z_N with generators g = 1 and h = s·g, plus an offset generator u = t·g. The accumulator starts at t·z via a lookup, so the first addition already has N distinct accumulator inputs. Then k windowed additions add the base-B digits of a and b times the matching table entries.
- **Failures.** Each distinct adder input (accumulator, digit) at each step fails independently with probability ε, as a fixed function of the input.
- **Failure kinds:**
    - wrong: output = correct + Δ, Δ random nonzero;
    - anc: output correct, an ancilla tag stays entangled;
    - phase: output correct and clean, sign flipped.

### 3.1 Formula check (brute force, N = 11, B = 2, k = 8, ε = 0.15)

| kind | closed form | brute-force DFT |
|---|---|---|
| none (ε = 0) | 1.000000 | 1.000000 |
| wrong | 0.231364 / 0.203183 | 0.231364 / 0.203183 |
| anc | 0.155386 / 0.172324 | 0.155386 / 0.172324 |
| phase | 0.073834 / 0.078205 | 0.073834 / 0.078205 |

(two seeds each; identical to six decimals)

### 3.2 Sweep at N = 101 (1,030,301 paths, 6 trials per cell)

Loss exponent r = −ln(P/P0)/(k·ε). Classical model: r = 1. Amplitude model: r = 2 (wrong, anc), r = 4 (phase).

| kind | B | k | ε = 0.005 | ε = 0.01 | ε = 0.02 |
|---|---|---|---|---|---|
| wrong | 2 | 14 | 1.61 | 1.75 | 1.67 |
| wrong | 4 | 8 | 1.52 | 1.73 | 1.81 |
| anc | 2 | 14 | 1.87 | 1.89 | 1.61 |
| anc | 4 | 8 | 1.75 | 1.72 | 1.82 |
| phase | 2 | 14 | 3.19 | 3.70 | 3.30 |
| phase | 4 | 8 | 3.57 | 3.00 | 3.65 |

Size sweep at ε = 0.01, B = 4 (wrong / anc / phase): N = 31: 1.91 / 1.51 / 3.47; N = 53: 1.74 / 1.78 / 3.29; N = 101: 1.62 / 2.01 / 3.47; N = 199: 1.81 / 1.84 / 3.57. No size gives r near 1.

### 3.3 Where the shortfall from 2 and 4 comes from

- **Per-line model.** line_check.py computes the per-line amplitude model, the mean over coset lines of (1 − h_v)², from the same random failures.
    - Phase: the simulation equals the per-line model to 4 decimals on all 4 seeds.
    - Wrong / anc: the simulation exceeds it by +0.004 to +0.024.
- **Source of the excess.** Paths that share one failing input receive the same offset Δ (or ancilla tag), so they stay mutually coherent. In this toy such a chunk is 1/B of a coset line at the first addition.
- **Residual vs chunk size** (resid_vs_B.py, N = 101, k·ε fixed at 0.08):

| B | chunk size at step 1 | mean residual (sim − model) |
|---|---|---|
| 2 | 1/2 of a line | +0.0077 ± 0.0013 |
| 4 | 1/4 | +0.0076 ± 0.0015 |
| 11 | 1/11 | +0.0025 ± 0.0001 |
| 101 | 1/101 | −0.0006 ± 0.0000 |

At B = 101 the residual equals the predicted O(δ/N) correction (−δ/N ≈ −0.0008). In real windowed secp256k1 Shor the first addition already has 2^32 distinct inputs. [a short passage is not captured] The asymptotic r = 2 and r = 4 therefore apply to the real attack, and the toy's lower values are a small-group artifact that makes the toy conservative.

## 4. Measured impact on the leader

HEAD 72ccdb8 (ops.bin SHA-256 prefix 7d3b49cbe63a8267), 128,000 shots on inputs independent of the op stream (two runs of 64,000, audit_eval.rs):

| class | run A | run B | total | rate | 95% UB |
|---|---|---|---|---|---|
| wrong output (W) | 133 | 123 | 256 | 0.2000% | 0.2218% |
| dirty ancilla only (A) | 0 | 0 | 0 | 0 | |
| phase only (P) | 23 | 23 | 46 | 0.0359% | 0.0459% |
| any | 156 | 146 | 302 | 0.2359% | 0.2595% |

Average executed Toffoli on these inputs: 743,567 and 743,570 (official: 743,575).

Pricing at k = 28 (point estimates):

| model | P_run | multiplier | score |
|---|---|---|---|
| none | 1 | 1 | 923.5M |
| classical (1−ε)^28 (P003 / Elliot) | 0.936 | ×1.068 | 986.7M (+6.8%) |
| amplitude (1−ε_W−ε_A)^56 (1−2ε_P)^56 (this claim) | 0.859 | ×1.165 | 1,075.5M (+16.5%) |

Under the classical model about one attack in 16 fails; under the amplitude model about one in 7. The gate quantity ε_W + ε_A + 2ε_P is 0.272%, above the 0.188% needed for 90% run success. So the leader fails the 90% gate under R4 while passing the classical gate (0.236% against 0.376%). [a short passage is not captured]

## 5. Proposed scoring (requirements)

Notation: k = 28 (Google 2603.28846, w = 16; ECDSA.Fail paper Eq. 11).

- **R1 (MUST)** Classify every shot into exactly one class, in this order: W (output mismatch), A (output correct, any non-register qubit nonzero at end), P (output and ancillas correct, phase bit set), OK. The current eval_circuit.rs counts output mismatches per shot, but phase and ancilla failures only per 64-shot batch, and it does not separate a phase failure from a shot that also has a wrong output. That is insufficient for R3.
- **R2 (MUST)** Compute one-sided upper confidence bounds ε_W^UB, ε_A^UB, ε_P^UB (Clopper-Pearson, as in R002/P003) on an input set independent of the op stream (fresh or secret per the C-TAIL/Track G discussion; this claim does not depend on which).
- **R3 (MUST)** Score: P_run = (1 − ε_W^UB − ε_A^UB)^(2k) · (1 − 2ε_P^UB)^(2k); Score = T̄·Q / P_run. [exact wording not captured]
- **R4** 90% gate ε_W + ε_A + 2ε_P ≤ 0.188%. [wording not captured]
- **Dirty resets.** [beginning not captured] … count a detected random-phase shot from a dirty R as class P (weight 2 in R3). By A.3 this prices dirty resets correctly without a separate rule. Mike's R002 proposal to flag every dirty reset deterministically is compatible; if adopted, count those shots as class A (weight 1) instead, which is the same expected cost.
- **Break-even effect.** Under P003, a change that saves a fraction ΔT/T of Toffolis while adding error Δε is profitable when ΔT/T > 28·Δε. Under R3 it must exceed 56·Δε for wrong outputs or ancillas and 112·Δε for phase errors. Changes in between are accepted by the classical score while making the real attack more expensive.

## 6. Limitations

- **Unstructured failures assumed.** The wrong outputs and ancilla garbage of different failing inputs do not coincide with good branches. Structured failures (for example, a failure that always outputs R + 2A) could keep partial coherence and cost less. [part not captured]
- **Early additions.** P003 already notes the model is weaker for the first additions of a run. The toy covers this case: its first addition has only N·B distinct inputs, and the model still holds.
- **P_ideal near 1 assumed** (true for Shor with standard post-processing). For algorithms with low ideal success, the cross term in A.2 is not negligible.
- **k = 28** is taken from the threads. The exponents scale linearly with k.

## 7. Agent verification tasks

Each task lists the command and the expected output. The doc states that the files are attached to the post; only this .md was visible in the thread.

- **V1. Formula check.** python3 brute_check.py (needs shor_error_toy.py in the same folder). Expect closed form and brute force identical to 6 decimals, and 1.000000 for ε = 0.
- **V2. Sweep.** python3 shor_error_toy.py 101 6 (about 3.5 minutes). Expect the table in 3.2. Check that no cell has r below 1.4 for wrong/anc or below 2.9 for phase.
- **V3. Per-line identity and residual.** python3 line_check.py, then python3 resid_vs_B.py. Expect phase diff = +0.0000 and the residual table in 3.3.
- [V4 and later not captured; the parallel reply in the P003 thread lists V4 (audit_eval.rs at 72ccdb8) and V5 (search for an unstructured failure with r ≈ 1 at c ≥ 2^16).]
