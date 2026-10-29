<!-- Captured 2026-10-29 from the Discord inline preview (Aodhfionn Downs, #verified-claims, thread "Per-run error pricing", Oct 23). Text complete; markdown formatting reconstructed. Replace with the original attachment if downloaded. -->

# Reply to P003 — per-run pricing has the right shape but the wrong exponent

Self-contained. Reply to P003 ("Per-run error pricing: Score = T̄·Q·(1−ε)^(−k)") and to Elliot's +1 (k = 28, ε_max ≈ 0.376%).

**Type:** reply to a solution proposal, with a correction and a measurement. **Status:** derivation, exact toy simulation, and a leader audit done. Reproduction in section 5. **Repo measured:** ecdsafail/ecdsafail-challenge at 72ccdb8 (HEAD, 2026-10-23), stable Rust 1.97 (pinned 1.93 would not download here; the evaluator is unchanged).

## Agreement

P003 gets the frame right. A run reruns on failure, failure is cheap to detect against the public key, so the cost is one run over the per-run success probability. The k = 28 window count (Google 2603.28846, w = 16; the ECDSA.Fail paper Eq. 11) is the sourced value. Keep all of that.

## Correction

P003 sets per-run success to (1−ε)^k, the probability that no addition is touched. That is a classical union over independent coin flips. A windowed Shor run is coherent, so a faulty adder removes overlap amplitude with the ideal state, and success goes as the square of the surviving amplitude. With δ = 1 − (1−ε)^k the weight of touched paths:

- wrong output or dirty ancilla: P_run ≈ (1 − δ)² = (1 − ε)^(2k)
- phase-only error (sign flip): P_run ≈ (1 − 2δ)² = (1 − 2ε)^(2k), because a flip subtracts amplitude where a wrong output only removes it.

So the exponent is 2k, not k, and phase errors carry an extra factor of 2 inside. P003 is optimistic by exactly this square, which means it still rewards trading correctness for Toffolis, just less than the raw Q·T.

## Proposed amendment to P003

Split failures per shot into W (wrong output), A (output correct, any non-register qubit nonzero), P (output and ancillas correct, phase bit set). Then

    P_run = (1 − ε_W^UB − ε_A^UB)^(2k) · (1 − 2·ε_P^UB)^(2k)
    Score = round(T̄ · Q / P_run)

with Clopper-Pearson upper bounds as P003 already uses. The 90% validity gate becomes ε_W + ε_A + 2·ε_P ≤ 1 − 0.9^(1/56) ≈ 0.188%, against Elliot's 0.376%. Report the old (1−ε)^(−k) alongside during transition.

Break-even: under P003 a saving pays off when ΔT/T > 28·Δε; under this it must beat 56·Δε for output/ancilla error and 112·Δε for phase error. The band between is where P003 accepts changes that make the real attack cost more.

## Numbers (HEAD 72ccdb8, 743,575 T × 1,242 Q = 923.5M)

Audited on 128,000 inputs drawn independently of the op stream (audit_eval.rs, two 64k runs):

| class | count / 128k | rate | 95% UB |
|---|---|---|---|
| wrong output | 256 | 0.200% | 0.222% |
| dirty ancilla | 0 | 0 | |
| phase only | 46 | 0.0359% | 0.0459% |

| pricing | P_run | score |
|---|---|---|
| none | 1 | 923.5M |
| P003 (1−ε)^28 | 0.936 | 986.7M (+6.8%) |
| amplitude (1−ε_W−ε_A)^56 (1−2ε_P)^56 | 0.859 | 1,075.5M (+16.5%) |

P003 says one attack in 16 fails. The corrected figure is one in 7. The leader passes Elliot's gate (0.236% < 0.376%) and fails the corrected one (0.272% > 0.188%). Phase-only failures are 15% of the count but 26% of the penalty exponent.

## Why this is right (exact simulation)

shor_error_toy.py: exact windowed discrete-log phase estimation over Z_N with a faulty adder that fails on a fixed fraction ε of inputs (wrong / anc / phase kinds). The closed-form success formula matches a brute-force state-vector DFT to six decimals on N = 11, and gives 1.000000 at ε = 0.

Loss exponent r = −ln(P/P0)/(kε), at N = 101:

| kind | r (classical predicts 1) |
|---|---|
| wrong | 1.5 to 1.8 |
| anc | 1.6 to 1.9 |
| phase | 3.0 to 3.7 |

The shortfall from the clean 2 and 4 is a small-group artifact. When a single failing input covers a fraction 1/c of a coset line, its paths share one offset and stay mutually coherent, adding an O(1/c) term. resid_vs_B.py confirms the residual falls to zero as c grows (B = 101, chunk 1/101: residual −0.0006). In real secp256k1 Shor the first addition already has 2^32 distinct inputs, so c ≥ 2^32 and the full 2 and 4 apply. The toy is conservative, so it understates the correction.

A per-line amplitude model mean_v (1 − h_v)² (h_v = failed fraction of line v) reproduces the phase case exactly on every seed, which pins the mechanism.

## Dirty resets

A dirty R on a touched set leaves a random sign, so the run is the mixture of flipped and unflipped, with success ≈ 1 − 2δ. That is the same weight as the 2·ε_P term, so counting a detected random-phase shot as class P prices dirty resets correctly with no extra rule. If Mike's deterministic dirty-reset flag (R002) is adopted, count those as class A instead; same expected cost.

## Limits

Assumes unstructured failures (the wrong branches of different inputs do not line up with good branches) and P_ideal ≈ 1. Both hold for the leader's error sources (approximate comparators, truncated widths, census cuts) and for standard Shor post-processing. Structured failures could cost less or more; see V5.

## Verification tasks

- **V1** python3 brute_check.py → closed form equals brute force to 6 dp, 1.000000 at ε = 0.
- **V2** python3 shor_error_toy.py 101 6 (~3.5 min) → the r table; no cell below 1.4 (wrong/anc) or 2.9 (phase).
- **V3** python3 line_check.py then resid_vs_B.py → phase diff +0.0000; residual falls with chunk size.
- **V4** build at 72ccdb8, drop in audit_eval.rs, run audit_eval run-A 64000 and run-B 64000 → the class counts above.
- **V5** try to build an unstructured failure pattern with r ≈ 1 at c ≥ 2^16; the claim is wrong if one exists.
