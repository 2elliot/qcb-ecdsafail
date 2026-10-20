# Reply — Track G thread: error composition, sample size, generator purity (v0.2)

2026-10-19 · Elliot D'Alessandro

## TL;DR and status

Track G is the right structure, and its fresh-input error pricing is what the Fiat-Shamir thread's fixed test set lacks. Three changes would make it bite:

1. **Compound error over the attack.** Price error over the K = 28 calls of a windowed attack, not per call: S_adj = Q·T / (1 − ε_UB)^K.
2. **Size the corpus.** Choose n_fresh and ε_max so the bound can actually separate current circuits (≈0.19%) from reference-quality ones (≈0.01%). That needs about 30,000 fresh inputs.
3. **Make the generator pure.** It reads no environment variables, files or clock; the corner block is drawn fresh.

- **Status:** DRAFT v0.2 (adds harness-audit evidence), a reply to `ecdsafail_modulus_specialization_AGENT_SPEC_v0_2.md`.
- **Verified:** every number here, recomputed in Python, including the spec's A8 reference value (0.0029912 for f = 0, n = 1000).
- **Not verified:** the contents of repo issues cited in R3 (titles only). The census and build-switch evidence in R1 and R3 comes from our harness audit; the λ contract is a submitter claim.

## Findings

Six findings; R1 and R2 change scores, R3 and R4 close loopholes, R5 and R6 are clarifications.

| # | Finding | Severity | Affects (v0.2 IDs) |
| --- | --- | --- | --- |
| R1 | Per-call pricing underprices errors that compound over 28 calls; the whole-attack penalty is about 28× larger | High | F8, F9 |
| R2 | With n_fresh = 10,000, a zero-failure circuit still gets ε_UB ≈ 0.03%, so reference-quality error (≈0.01%) cannot be resolved | Medium | §4 n_fresh, F7, OQ1 |
| R3 | S3 limits the generator's inputs but not its environment; repo issues show build behavior steered by environment variables | Medium | S3, A16 |
| R4 | A fixed or public corner block is patchable at about 46 Toffolis per failing point (Fiat-Shamir reply, F2) | Medium | §5.2 |
| R5 | P-256's fold constant is 224 bits wide but has signed-digit weight 4, so the bitlen and weight thresholds disagree | Low | F2, A3, OQ6 |
| R6 | State that secp256k1 specialization is legitimate for Track S, and leave room for an interface parameter in `build` | Low | §1, F1 |

## R1: Price error over the whole attack, not per call

F8's S_adj = Q·T / (1 − ε_UB) assumes one point addition can be retried on its own. In Shor, about 28 additions run inside one coherent computation, and the whole run is repeated if any of them fails. The cost per successful attack therefore scales with (1 − ε)^−K.

```math
S_{\mathrm{adj}} = \frac{Q \cdot T}{(1-\varepsilon_{\mathrm{UB}})^{K}}, \qquad K = 28 \;(\text{windowed},\ w = 16)
```

The literature already composes this way. The trapped-ion estimate union-bounds approximate-arithmetic failures over its 28 additions. A recent write-up on the 893-qubit leaderboard circuit converts "one failure in 3,250 additions" into under 1% of attack runs.

| Error rate ε | Per-call penalty 1/(1 − ε) | K = 28 penalty | K = 512 (double-and-add) |
| --- | --- | --- | --- |
| 0.19% (best circuit today) | 1.0019 | 1.055 | 2.65 |
| 0.5% (v0.2's placeholder ε_max) | 1.005 | 1.151 | 13.0 |

K depends on the Shor interface, so the rules should fix it explicitly. A separate claim on the addend interface argues the windowed one (K = 28) is the right target.

The leaderboard now allows approximation explicitly, which makes this pricing urgent.

- **The λ contract.** Recent PRs cite a "Circuit Lambda" limit of 18 expected failure events per 9,024-shot run (PR 419; a submitter claim). That is ε ≈ 0.2%, or about a 5.8% whole-attack penalty at K = 28.
- **Empirical gate stripping.** Our harness audit found the incumbent removes 9,678 Toffolis because they never fired in a census of 121.6M random input pairs (`deep_strip_keys.rs`). That is an empirical approximation, which only fresh-input pricing will catch.

## R2: n_fresh and ε_max must match what the bound can resolve

At n_fresh = 10,000, a perfect circuit still gets ε_UB ≈ 0.030%. That is three times the reference circuits' ≈0.0099% (2^−13.3). Certifying reference-level error with zero failures needs n ≥ ln(0.05) / ln(1 − 2^−13.3) ≈ 30,200 fresh inputs.

One-sided 95% Clopper–Pearson upper bounds at n = 10,000:

| Failures f | ε_UB | Per-call penalty | K = 28 penalty |
| --- | --- | --- | --- |
| 0 | 0.030% | 1.0003 | 1.008 |
| 1 | 0.047% | 1.0005 | 1.013 |
| 5 | 0.105% | 1.0011 | 1.030 |
| 19 (≈ today's best, 0.19%) | 0.279% | 1.0028 | 1.081 |
| 50 | 0.632% | 1.0064 | 1.194 |

The placeholder ε_max = 0.5% is 2.6× today's best and about 50× the references, so the gate rarely bites. Two principled options for OQ1:

- **Option A, tie ε_max to attack success.** Require (1 − ε_UB)^28 ≥ 0.95, i.e. ε_max ≈ 0.183%. Today's best (ε_UB 0.279% at n = 10,000) would fail this gate, a deliberate push toward exactness.
- **Option B, tie ε_max to a 90% bar.** Require (1 − ε_UB)^28 ≥ 0.90, i.e. ε_max ≈ 0.376%. Current circuits pass, and R1's K = 28 pricing still rewards lowering error.

Under either option, raise n_fresh for ranked instances to at least 30,000 if T9's benchmark allows.

## R3: The generator must be a pure function

S3 limits what the generator receives (`ModulusParams` plus helpers), but not what it can read on its own. The live repo shows why that matters. Open issue titles report generator behavior steered by environment variables, including:

- `DROP_DEAD_FILE` paths silently falling back to embedded lists;
- `DROP_DEAD_ROBUST_DISABLE` not fully disabling a drop pass;
- `benchmark.sh` ignoring environment variables on one path;
- peak Q being "set by two environment values".

Our harness audit adds source-level evidence. 3616dbf2's default build takes its ping-pong route unless `SUB4_LEGACY_POINT_ADD` is set (`mod.rs:2525-2541` at `897dda2b`). The current top lineage's arithmetic layout is selected by named flags such as `LEAPFROG`, `LF_SEED` and `LF_SEAMS` (`leapfrog.rs:306, 309, 374`). An empty-environment build exposes any submission whose score depends on such switches.

Proposed S3′: the generator is a pure function of `ModulusParams` (and the interface parameters, see R6).

- No environment variables, no files outside its own source tree, no clock, no network, no randomness except a seed the harness passes in.
- The harness builds in a sandbox with an empty environment, a read-only source tree, and no network.
- A16′ extends A16: build the same (p, n) twice under different environments (random variables set, different working directory) and require byte-identical op streams.

Embedded drop lists in source stay allowed, since they are code. Hidden primes already defeat lists tuned to secp256k1.

## R4: Draw the corner block fresh, randomized within each class

§5.2's corner block (x_R − x_A ∈ {1, 2, p − 1, p − 2, small, ~p/2}, extremal coordinates, limb boundaries, alternating masks) is good coverage. If those vectors are fixed, or derived from public data, they are patchable at about 46 Toffolis per failing point. That construction is F2 of the Fiat-Shamir reply.

- **Fresh seed.** Derive the corner block from the post-window epoch seed, like the fresh inputs (S1).
- **Random members per class.** Pick random instances within each class, for example random x_A with x_R = x_A + 1, rather than one canonical vector.
- **Count every trial.** Corner failures count in eps_UB with the rest. Optionally report them separately, since they flag structural rather than random errors.

## R5: P-256 splits the analyze_modulus thresholds

For A3: P-256's fold constant is wide but sparse, so the bit-length and weight tests give opposite answers. With p = 2^256 − 2^224 + 2^192 + 2^96 − 1:

```math
F = 2^{256} - p = 2^{224} - 2^{192} - 2^{96} + 1
```

| Modulus | bitlen(F) | Signed-digit (NAF) weight of F |
| --- | --- | --- |
| secp256k1 | 33 | 5 |
| P-256 | 224 | 4 |

With a small B_f (say 40), `analyze_modulus(P-256)` returns None, yet Solinas reduction for P-256 is cheap. Suggest that `FoldInfo` report bitlen and weight separately instead of a single Some/None. Then A3 pins two measured numbers, and OQ6 decides which threshold the rules use.

## R6: Framing, OQ2, and an interface parameter

- **Framing.** Say plainly in §1 that specializing to secp256k1 is legitimate when the goal is attacking Bitcoin; Track S keeps measuring that. Track G measures reusability. Without this, readers will object that Track G "punishes legitimate optimization".
- **OQ2.** Agree with amending the commit-reveal spec's zero-failure rule to an ε_UB rule; R1's K = 28 pricing is a principled middle ground. Zero failures over 100,000 inputs would disqualify today's top circuits, which fail about 0.19% of the time.
- **Interface parameter.** `build(&ModulusParams)` should leave room for interface parameters such as a window width w and how the addend is supplied (classical vs quantum-addressed). A separate claim argues the scored kernel should take the addend through a quantum-addressed table lookup, since that is what windowed Shor runs. Track G and that track would then share one signature.

## Proposed amendments to v0.2

Each row edits one v0.2 requirement; IDs match the spec.

| v0.2 ID | Current | Proposed |
| --- | --- | --- |
| F8 | S_adj = Q·T / (1 − ε_UB) | S_adj = Q·T / (1 − ε_UB)^K, with K = 28 fixed in the rules |
| §4 n_fresh | ranked 10,000; probe 3,000 | ranked ≥ 30,000 if T9 allows; probe unchanged |
| §4 eps_max | 0.005 placeholder | Option A (0.183%) or Option B (0.376%), from R2; decide in OQ1 |
| S3 | Generator gets only `ModulusParams` + helpers | S3′: pure function; empty-env, read-only, no-network build sandbox |
| A16 | Same (p, n) twice → identical output | A16′: same (p, n) under two different environments → byte-identical output |
| §5.2 | Corner block from independent XOF labels | Corner block from the post-window epoch seed, randomized within each class |
| F2 | Some iff bitlen ≤ B_f and weight ≤ W_f | `FoldInfo` reports bitlen and weight separately; A3 pins both for P-256 |
| §1 | — | Add the Track S legitimacy sentence (R6) |

## Verification tasks for agents

Each finding has a check; T-R1 and T-R2 are pure math and can run anywhere.

| Task | What to do | Pass condition |
| --- | --- | --- |
| T-R1 | Recompute 1/(1 − ε)^K for ε ∈ {0.19%, 0.5%} and K ∈ {1, 28, 512} | Matches the R1 table to 3 significant figures |
| T-R2 | Implement the §5.3 bisection; compute ε_UB at n = 10,000 for f ∈ {0, 1, 5, 19, 50}; solve for n where f = 0 gives ε_UB = 2^−13.3 | Matches the R2 table; n ≈ 30,200 |
| T-R3 | Grep the challenge repo for `std::env`, `env::var`, file reads and time calls inside `src/point_add/`; read the four issues cited in R3 | file:line list; confirm or correct each issue summary |
| T-R3b | Implement A16′: build twice under different environments and compare op-stream hashes | Hashes identical |
| T-R4 | Run the Fiat-Shamir reply's T-F2 patch against a fixed corner block | The patch passes the fixed block; fails the fresh block at the expected rate |
| T-R5 | Compute F, bitlen(F), and the NAF weight for secp256k1 and P-256 from the standard p | Matches the R5 table |

## Open questions

1. Should K be 28 (windowed) for all of Track G, or vary with the declared interface?
2. Which ε_max option: A (0.183%, pushes exactness) or B (0.376%, current circuits pass)?
3. Can T9's runtime budget afford 30,000 fresh inputs per ranked instance?
4. Should corner-block failures be reported separately from bulk failures?

## Sources

- QCB harness audit, `audit_report.md` (attached to the classical-addend claim thread): census-based gate stripping, build-route switches, the PR 419 λ contract
- [ECDSA.Fail paper, arXiv:2609.09582](https://arxiv.org/pdf/2609.09582): 0.192% failure on 100,000 independent inputs; the Q×T/p̂ retry proxy; reference error ≤ 2^−13.3; 28-call windowed schedule
- [Trapped-ion ECDLP estimate, arXiv:2609.05625](https://arxiv.org/html/2609.05625): union bound over 28 point additions
- [PostQuantum on the 893-qubit circuit](https://postquantum.com/security-pqc/ecdsa-fail-low-qubit-circuit/): per-addition failure rate converted to attack-run failure
- [Repo issue list](https://github.com/Layr-Labs/ecdsafail-challenge/issues): environment-variable and drop-list issues (titles only)
- Thread under reply: `ecdsafail_modulus_specialization_AGENT_SPEC_v0_2.md`; companion: Fiat-Shamir reply v0.2
