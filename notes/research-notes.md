# Research notes

Durable facts collected during the week of 2026-10-19. Labels: **measured** (our audit), **source-read**, **claimed** (by submitters or authors), **computed** (recomputed by us in Python). The leaderboard moves fast, so re-check anything dated before relying on it.

## 1. Harness facts (Layr-Labs/ecdsafail-challenge at `3dee0cd6`; measured / source-read)

Harness files are unchanged since `c66c9586` (2026-06-22).

| Fact | Where |
| --- | --- |
| Score S = Q × T. Q = highest qubit index + 1 (static, not live-peak); T = average executed Toffolis per shot | `circuit.rs:355-363`; `eval_circuit.rs:357-361` |
| 9,024 test points derived by Fiat-Shamir from a hash of the op stream; submitters grind a 48-bit nonce encoded as a 96-op X tail | `eval_circuit.rs`; paper §3.3 |
| Offsets (the addend A) are classical bits in registers 2 and 3 | `eval_circuit.rs:471-508` |
| Ops can be classically conditioned (`c_condition` or Push/PopCondition stack); a CCX/CCZ counts only when its condition holds | `sim.rs:73-87` |
| X, Z and classical bit ops (BitInvert, BitStore) are free; with the condition stack they give free classical logic | `sim.rs:95-98`, `156-164` |
| Classical bits are never checked at the end | `eval_circuit.rs:326-340` |
| Test filter removes A = O, R = O, R = ±A (R = −2A is not filtered) | `eval_circuit.rs:257-265` |
| Measurement outcomes come from the same Fiat-Shamir XOF as the inputs (8 bytes per R/HMR) | `eval_circuit.rs:273` |
| The README lists a forward∘reverse identity check, but HEAD does not implement it; the scorer runs one forward pass | audit V1, conflict C3 |

## 2. Circuits and leaderboard (as of 2026-10-19)

| Circuit | Q | T | S | Notes |
| --- | --- | --- | --- | --- |
| 8e9c9a22 (paper's incumbent, `60d61859`) | 1,151 | 1,299,452.7 | 1.496 × 10^9 | measured; 0.192% any-channel failure on 100k independent inputs (paper) |
| 3616dbf2 (ping-pong GCD, `897dda2b`) | 1,321 | 952,707.5 | 1.259 × 10^9 | measured |
| 3493c837 (lowest-Q promoted with PR metrics, `31f9c58c`) | 1,150 | 1,284,776.1 | 1.477 × 10^9 | measured |
| Rank 1: 54fc4f25 (PR #817) | 1,234 | 761,052 | 939,138,168 | claimed |
| Rank 2: d9ae2845 (PR #814) | 1,234 | ~761,077 | 939,169,018 | bot-recorded S |
| Rank 3: 9dc3798c (PR #718) | 1,234 | ~761,096 | 939,192,464 | bot-recorded S |

- **Low-qubit branch.** The sub-900-qubit circuits (813–825 Q in the paper; a Q793 validation PR) are not on the promoted list, and they have enormous T.
- **Approximation contract.** Recent PRs cite a "Circuit Lambda" limit of 18 expected failure events per 9,024-shot run (PR 419, claimed), i.e. ε ≈ 0.2%.
- **Empirical gate stripping.** The incumbent removes 9,678 Toffolis that never fired in a 121.6M-sample census (`deep_strip_keys.rs`; measured).
- **Paper's windowed-compatible variant of 8e9c9a22** (claimed): Q 1,162, T 1,684,161, i.e. +11 Q and +384,708 T (+29.6%). Its code is not in the challenge repo.

## 3. Literature

- **ECDSA.Fail paper**, [arXiv:2609.09582](https://arxiv.org/pdf/2609.09582): challenge design, landability, nonce tail, task domain (R ∉ {A, −A, −2A}), support-calibrated widths, windowed variant (§5.2.3, Supplemental Note F).
- **Babbush et al.**, [arXiv:2603.28846](https://arxiv.org/html/2603.28846v2): w = 16, 28 windowed additions, measurement-based uncomputation of lookups. Their ZK proofs cover the same 9,024-input harness.
- **Schrottenloher**, [arXiv:2606.02235](https://arxiv.org/html/2606.02235v1): per-addition lookup accounting of +16 qubits and 3·2^w Toffolis.
- **Trapped-ion ECDLP estimate**, [arXiv:2609.05625](https://arxiv.org/html/2609.05625): random offset masks so tables avoid O; union bound over 28 additions.
- **PostQuantum** on the 893-qubit circuit: [article](https://postquantum.com/security-pqc/ecdsa-fail-low-qubit-circuit/).

## 4. Formulas and computed numbers

**Landability** (pass all N = 9,024 hash-derived tests):

```math
\Pr[\text{pass}] = (1-\varepsilon)^{9024}
```

- ε = 0.19% → 2^−24.8; 0.37% → 2^−48.3; 1% → 2^−130.8; 15% → 2^−2116.
- With 2^W nonce work, the maximum hideable ε is 1 − 2^(−W/9024): about 0.25% (W = 32), 0.31% (W = 40), 0.37% (W = 48).

**Error composed over a windowed attack** (K = 28 additions):

```math
S_{\mathrm{adj}} = \frac{Q \cdot T}{(1-\varepsilon_{\mathrm{UB}})^{K}}
```

- ε = 0.19% → ×1.055 (K = 28), ×2.65 (K = 512). ε = 0.5% → ×1.151, ×13.0.
- ε_max for 95% attack success: 0.183%. For 90%: 0.376%.

**One-sided 95% Clopper–Pearson upper bounds:**
- f = 0, n = 1,000 → 0.0029912.
- At n = 10,000: f = 0 → 0.030%, 1 → 0.047%, 5 → 0.105%, 19 → 0.279%, 50 → 0.632%.
- Certifying ε ≤ 2^−13.3 with zero failures needs n ≈ 30,200.

**Windowed-adjusted score** (k = addend-access windows; 512 is an inferred measurement-based unlookup cost):

```math
S_W = (Q + 16)\,\bigl(T + k\,(2^{16}-2) + 512k\bigr)
```

- Incumbent (k = 3): 1.748 × 10^9.
- Leader: 1.199 × 10^9 (k = 3) to 1.364 × 10^9 (k = 5). With +256 qubits of seam width: 1.445–1.643 × 10^9.
- Leaderboard progress since the incumbent is 37.2% in S, but only 6–31% in S_W.

**Lookup traversal cost:** 65,534 T at w = 16 (PR #15 construction, measured), so the paper's +384,708 ≈ 5.87 traversals.

**P-256 fold constant:** F = 2^256 − p = 2^224 − 2^192 − 2^96 + 1. bitlen 224, NAF weight 4. For secp256k1: F = 2^32 + 977, bitlen 33, NAF weight 5.

## 5. Teammate claims (as received, in `context/teammate-claims/`)

1. **Fiat-Shamir / robust architecture** (`robust_architecture_specification.md`): canonicalizer; 512 corner points + 10,000 fixed public points; exact cls/anc/pha + forward∘reverse checks.
2. **Modulus specialization / Track G** (`ecdsafail_modulus_specialization_AGENT_SPEC_v0_2.md`): parametric `build(p, n)`; ranked family (secp256k1, P-256, hidden sparse and dense primes); n = 192/224/384 probe; error-priced score with a Clopper–Pearson bound; geomean.
