# Reply — Fiat-Shamir thread: fixed public test sets are patchable (v0.2)

2026-10-19 · Elliot D'Alessandro

## TL;DR and status

The thread's diagnosis of Fiat-Shamir nonce grinding is right, and our harness audit confirms it. Top submissions pin a ground 48-bit tail nonce in code, and the incumbent's own notes expect about 10 output mismatches per 9,024-shot run. Its replacement fails cheaply, though: a fixed, public 10,512-point test set can be passed by patching each failing public point for about 46 Toffolis. A 1%-error circuit then passes for under 0.4% of T.

Proposed fix: keep the diagnosis and the per-shot exact checks. Draw every test input, including the corner cases, fresh and secret per evaluation, as the Track G thread already specifies. Drop the canonicalizer.

- **Status:** DRAFT v0.2 (corrects F4; adds harness-audit evidence), a reply to `robust_architecture_specification.md`.
- **Verified:** all arithmetic here, recomputed in Python. Statements from the challenge README and the ECDSA.Fail paper (Sources).
- **Not verified:** anything not covered by our harness audit at commit 3dee0cd; F4 and F6 were checked against the code. The patch in F2 is a construction argument, not yet a built circuit; task T-F2 builds it.

## Findings

Six findings, ordered by section. F2 is the one that changes the design.

| # | Finding | Severity | Evidence | Proposed change |
| --- | --- | --- | --- | --- |
| F1 | A 15% failure rate cannot pass 9,024 hash-derived tests; a 48-bit nonce hides at most ~0.37% | Low (factual fix) | Landability (1 − ε)^9024, paper §3.3 | Correct the figure or cite its source |
| F2 | Fixed public test points can be patched for ~46 Toffolis each | High | Construction in F2 | All inputs fresh and secret per evaluation |
| F3 | The canonicalizer is redundant once inputs ignore the op stream, and incomplete anyway | Medium | `X(a) X(b) X(a) X(b)` survives it | Drop it, or normalize for reporting only |
| F4 | Ancilla and phase checks already exist; the README's forward-reverse check is not implemented, and it is trivially true for unitary gate lists anyway | Low | Harness audit at 3dee0cd: the scorer runs one forward pass | Keep the adjoint pass as new work, scoped to HMR/reset; measure LOC and runtime |
| F5 | Requiring complete adders conflicts with the task domain and the literature; the R = −2A case is missing | Medium | Paper §3.1; offset masks in arXiv:2609.05625 | Declared domain plus random offset masks |
| F6 | The measurement-randomness seed is unspecified; a public seed makes rare dirty measurements landable | Medium (conditional) | Spec §4.2 passes `rand_xof` unseeded | Secret per-evaluation seed |

## F1: The 15% figure

A 15% failure rate is unhideable; the realistic ceiling is about 0.37%. Under random-oracle test selection, a circuit with per-input error ε passes all N = 9,024 tests with the probability below. The paper calls this landability (§3.3).

```math
\Pr[\text{pass}] = (1-\varepsilon)^{9024}
```

| Error rate ε | Landability | Expected nonces to land |
| --- | --- | --- |
| 0.19% (best circuit on 100k independent inputs) | 2^−24.8 | ~3 × 10^7 |
| 0.37% | 2^−48.3 | the whole 48-bit space |
| 1% | 2^−130.8 | infeasible |
| 15% | 2^−2116 | infeasible |

The figure should be corrected unless "15%" refers to another input distribution, such as corner cases only. The real exploit is hiding about 0.2% error, which still matters once errors compound over 28 calls (see the Track G reply).

The leaderboard's own rules point the same way. Recent PRs cite a "Circuit Lambda" contract limit of 18 expected failure events per 9,024-shot run (PR 419; a submitter claim, not verified). That is ε ≈ 18 / 9,024 ≈ 0.2%: landable by grinding, and well short of 15%.

## F2: Fixed public test points can be patched cheaply

The economic proof assumes overfitting means memorizing all 10,512 points (over 5 × 10^6 Toffolis). A contestant only needs to fix the few public points their approximate circuit gets wrong. With a public set, they know exactly which ones those are.

**Construction**, one patch per failing public point k, appended at the end of the circuit:

1. **Pick distinguishing bits.** Choose b output bits that separate the wrong output W_k from every other public output, and likewise for the correct output C_k. With b = 24, a collision with another public point has probability about 6 × 10^−4 per patch; the contestant checks and re-picks.
2. **Raise a flag.** Compute the flag "those b bits equal W_k's": X gates on the zero bits, then a b-input AND. Cost: b − 1 Toffolis on temporary ancillas.
3. **Correct the output.** Controlled on the flag, XOR D_k = W_k ⊕ C_k into the output registers. CNOTs only, no Toffolis.
4. **Clear the flag.** Uncompute it by testing b output bits against C_k's pattern. Cost: at most b − 1 Toffolis, less with measurement-based AND uncomputation.

Cost per patch is about 2(b − 1) = 46 Toffolis. It needs a few temporary qubits at the end of the circuit, away from the peak. On the public set it is value-, ancilla- and phase-clean. On fresh inputs it misbehaves with probability about 2^−24 per patch.

| Circuit error ε | Failing public points (of 10,512) | Patch cost (Toffolis) | Share of T = 1.30M |
| --- | --- | --- | --- |
| 0.19% | ~20 | ~920 | 0.07% |
| 0.5% | ~53 | ~2,400 | 0.19% |
| 1% | ~105 | ~4,800 | 0.37% |

Any approximation that saves more than its patch cost becomes profitable: truncated carries, a shorter GCD budget, narrowed widths. The paper reports exactly such knobs, for example GCD budgets of 258 vs 261 macro-steps traded against non-convergence rate. The repo already has an open issue reporting an off-sample counterexample for a promoted submission, so off-sample failures are real.

The same attack applies to the hardcoded Tier 1 corner vectors. Even without patching, a fixed set allows knob grinding: searching functionally different variants until one passes, the same search as nonce grinding.

**Unverified, depends on the harness:** if gates can be classically conditioned on the addend bits, a patch could fire only on its own test shot. Its average executed cost would then be close to zero. The pending V1 audit settles this.

The fix is that inputs must be unknown when the circuit is frozen: drawn from a seed revealed after the submission window, as Track G §5.1 already specifies.

## F3: The canonicalizer is redundant and incomplete

Once test inputs no longer depend on the op stream, op-stream malleability changes nothing, so the canonicalizer adds no integrity. As a defense it would also be weak:

- **Adjacent pairs only.** It cancels only adjacent identical self-inverse pairs. `X(a) X(b) X(a) X(b)` survives, as do commuting reorders, relabeled ancillas, and gates on fresh scratch qubits.
- **False rejections.** `MALLEABLE_PADDING_DETECTED` rejects more than 4 consecutive cancellations. Generated code with mirrored compute/uncompute ladders can produce such runs legitimately.
- **Score drift.** Cancelling `CCX;CCX` pairs changes reported T. That conflicts with any rule that existing (Track S) scores stay bit-identical.

Recommendation: drop it. If kept, use it only to normalize reported op counts, never to reject.

## F4: Two of the three exact checks already exist

The challenge README lists forward-reverse identity as a validity check, but the harness at commit 3dee0cd does not implement it. The scorer runs one forward pass, then checks outputs, phase, and the final state of non-register qubits. v0.1 of this reply said the pass already existed; that was wrong, and Pillar 3's adjoint pass is genuinely new work.

- **Ancilla and phase checks: retained.** They already run on every shot in `eval_circuit.rs`.
- **Forward-reverse: new, but narrow.** For any gate list built only from X, CX, CCX, SWAP and diagonal phase gates, running the reversed adjoint always restores the input. The pass adds information only where HMR or reset ops appear, so scope it to those rather than calling it an "absolute proof of reversibility".
- **Reference code needs porting.** `Op::adjoint` and `is_abi_register` do not exist in the harness, so the spec's §4 code would not compile as written.
- **Re-measure.** Share the commit and hardware behind "+138 lines" and "~4.5 s", or measure them.

## F5: Exceptional cases: declare the domain and use offset masks

Requiring complete adders raises every call's cost to cover events Shor almost never hits. It would also make the benchmark incomparable to published estimates.

- **Nothing was stripped.** The task defines the map only for R, A ≠ O and R ∉ {A, −A, −2A}. The evaluator filters identity inputs and x_R = x_A (paper §3.1). The baseline never handled doubling or negation, so no submission removed them.
- **Tier 1 misses R = −2A.** That case gives x_A − x_R′ = 0, which breaks the in-place multiplication the circuits use.
- **Shor handles exceptions with random offsets.** The trapped-ion estimate starts the accumulator at a random nonzero point and adds a random offset point to every table entry, so tables never contain O. Exceptional cases then occur at random inputs with probability around 1/r ≈ 2^−256 per addition and are union-bounded.
- **Recommendation.**
    - Declare the domain explicitly, including −2A, and generate inputs from it.
    - In a windowed interface, use offset masks rather than identity-row or doubling logic.
    - If the group wants completeness, make it an opt-in track rather than a gate on the main score.

## F6: Measurement randomness must be fresh and secret

This finding is conditional on how the proposed simulator is seeded, which the spec does not say. Our harness audit (commit 3dee0cd) shows the current scorer reuses the Fiat-Shamir XOF for measurement outcomes, reading 8 bytes per R or HMR. Kept unchanged under a static public seed, those outcomes would be fixed and public. The simulator injects pseudo-random bits when HMR or reset hits a dirty qubit (spec §2.3), and §4.2 passes `rand_xof` without stating its seed.

- **The risk.** If those bits come from the public seed, measurement outcomes are known in advance. Take a circuit that leaves an ancilla dirty on a rare branch and then measures it. It is caught only when the injected bit is 1 on an affected shot, so the contestant can reorder measurements until those bits are 0.
- **Requirement.** Measurement randomness MUST come from a secret per-evaluation seed, domain-separated from input generation.

## Proposed merged design

The two threads combine cleanly. This thread keeps its diagnosis and corner-case coverage; Track G supplies fresh inputs and error pricing. MUST/SHOULD are per RFC 2119.

| ID | Requirement | Replaces in this spec |
| --- | --- | --- |
| M1 | All ranked test inputs MUST be unknown when the circuit is frozen, derived from a seed revealed after the submission window (Track G §5.1, S1) | Tier 2 static public seed |
| M2 | The corner tier MUST be randomized within each class and drawn fresh. Classes include x_R − x_A ∈ {1, 2, p − 1, p − 2}, random values at each 64-bit limb boundary, and reduction-interval operands | Tier 1 hardcoded vectors |
| M3 | Failures MUST be counted and priced (Track G F6–F8), not judged zero-tolerance on a small set | All-or-nothing validation |
| M4 | Measurement randomness MUST come from a secret per-evaluation seed | Unspecified `rand_xof` |
| S1 | The canonicalizer SHOULD be dropped, or used only to normalize reported op counts | Pillar 1 |
| S2 | The input domain SHOULD be declared (R, A ≠ O; R ∉ {A, −A, −2A}), with offset masks replacing complete-adder demands | Tier 1 singularity vectors |
| S3 | The existing cls/anc/pha and forward-reverse checks SHOULD stay as they are | Pillar 3 (relabel only) |
| S4 | A public dev seed SHOULD remain available for local iteration, never used for ranking | Local reproducibility goal |

S4 keeps this thread's local-reproducibility goal. Contestants iterate on a public dev set, and official rankings become reproducible once the seed is revealed.

## Verification tasks for agents

Each finding has a check an agent can run. T-F2 is the one that matters; the rest confirm facts.

| Task | What to do | Pass condition |
| --- | --- | --- |
| T-F1 | Recompute (1 − ε)^9024 for ε ∈ {0.19%, 0.37%, 1%, 15%} | Matches the F1 table to 0.1 in the log2 exponent |
| T-F2 | Take any passing circuit. Corrupt its output on k = 5 chosen public points. Append the F2 patch with b = 24 | Passes every public point with cls/anc/pha = 0/0/0; added Toffolis ≤ 2(b − 1)k; on 10^5 fresh inputs, extra failures ≤ k × 2^−24 × 10^5 |
| T-F3 | Append `X(a) X(b) X(a) X(b)` to a circuit and run the canonicalizer | It survives canonicalization and changes the op-stream hash |
| T-F4 | Locate the forward-reverse check in the current harness; time the existing evaluator | file:line found; runtime and LOC stated with commit and hardware |
| T-F5 | Check whether the evaluator's input filter excludes R = −2A | file:line; confirm or deny |
| T-F6 | Find how the simulator's measurement randomness is seeded | file:line; is the seed public before evaluation? |

T-F2 can be built on a toy bit-level simulator if the harness is unavailable. It only needs X, CX, CCX and the public output values.

## Open questions

1. What is the source of the 15% figure, and on which input distribution?
2. What were the commit and hardware behind the LOC and runtime numbers?
3. Does the group want complete addition as a separate opt-in track?
4. Who holds the evaluation seed, and when is it revealed? (Track G OQ7)

## Sources

- QCB harness audit, `audit_report.md` (attached to the classical-addend claim thread): V1 mechanism at commit 3dee0cd, tail-nonce pinning, measurement seeding, the PR 419 λ contract
- [ecdsafail-challenge README](https://github.com/Layr-Labs/ecdsafail-challenge): validity criteria (stale on forward-reverse, per the audit)
- [ECDSA.Fail paper, arXiv:2609.09582](https://arxiv.org/pdf/2609.09582): §3.1 task domain and evaluator filter; §3.3 nonce tail and landability; §5.2.2 support-calibrated widths; §5.3.1 GCD budget vs non-convergence
- [Trapped-ion ECDLP estimate, arXiv:2609.05625](https://arxiv.org/html/2609.05625): random offset masks, union bound over 28 additions
- [Repo issue list](https://github.com/Layr-Labs/ecdsafail-challenge/issues): off-sample counterexample report for a promoted submission
- Thread under reply: `robust_architecture_specification.md`; companion: `ecdsafail_modulus_specialization_AGENT_SPEC_v0_2.md`
