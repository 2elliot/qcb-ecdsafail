# Next steps and candidate claims

## Follow-ups (priority order)

0. **Build the zero-cost public-corpus demo.** This is the most important open item, because positions-list item 3 in `replies/drafts/README.md` rests on it.
    - **Claim (inferred from audit V1, not built).** Under the classical-addend ABI, a circuit can recognize any publicly known test input for free: classical bit ops are free, stacked conditions AND together, and condition-false Toffolis aren't counted. On a public cost corpus it can switch off all arithmetic and XOR in precomputed answers, giving T̄ ≈ 0 and score ≈ 0. On a public error corpus it can patch every failure for 0 Toffolis.
    - **Why today's harness is immune.** Inputs are hashed from the op stream, so the corpus can't be known before the circuit is fixed.
    - **Demo plan.** Use only harness code and our own tooling; no new contestant builds.
        1. Fix a public seed and derive a small corpus (e.g. 64 points), as in Mike's `pfs fixed`.
        2. Write a transformer over the already-built incumbent `ops.bin`. It prepends a classical recognizer (BitInvert / PushCondition / BitStore1 per corpus point), wraps every existing op in PushCondition(¬flag) … PopCondition, and appends conditioned X gates that write the precomputed R_i + A_i.
        3. Evaluate with a fixed-seed version of the scorer. Expected: corpus shots all correct, executed T ≈ 0 on them. Fresh random inputs: still correct, T unchanged.
    - **Watch for.** The interaction of the wrapper condition with existing measurement-conditioned blocks (conditioned R/HMR are allowed per Joel's §3.5); op-count growth.
1. **Fold the demo into our docs.** Claim v0.2 → v0.3 (a stronger argument for the quantum boundary), and Fiat-Shamir reply → v0.3 (F2 becomes "fixed public sets are zeroable", not just "patchable at ~46 T").
2. **AT2/AT3 for the claim (sandboxed).** Leader's k and division-seam width; settles the 6–31% range in claim v0.1.
3. **Incumbent fresh-input failure rate.** Rewrite only the 96 tail X targets; verifies ~0.19%. Mike's R002/P003 already measured 0.173% wrong-output / 0.208% any-channel on 7499c90, so this mainly cross-checks his tool.
4. **Ask the paper's authors** for the Supplemental Note F (windowed variant) code.
5. **Build T-F2** (the patch construction) on a toy simulator. This is now subsumed by item 0.
6. **Pending replies.** Six drafts in `replies/drafts/`, held until more teammates respond.

## Open questions carried forward

- **R2 (offline-exact scoring).** Is it a hard requirement (Mike) or negotiable (Joel's post-commit rounds)? Our view: with classical addends it is incompatible with integrity; with a quantum addend it needs N_err ≳ 4 × 10⁵.
- **Track G.** Ranked (Joel's synthesis) or an optional research track (Mike)? Soft-floor ratio?
- **Default switch** to a quantum-addend track: needs a passing reference circuit, a decoupled sampler, and published constants.
- **The leader's k (3–5) and seam width (0 to +256 qubits)**; the current leader is now 7499c90.
- **Unlookup cost** to use in scores: the literature's ~512 T, the repo's experimental 32,766 T, or replay at 65,534 T.
- **k = 28 vs 32** (we hold 28); and whether (1 − ε)^−k is a ranking rule or only a diagnostic.

## Candidate future claims

- **Public-corpus zeroing under classical addends.** It could be its own claim once item 0 is built.
- **Support-calibrated approximation + scale-model verification.** Calibrated widths, census-mined gate stripping, and fitted walk-width tables (Mike verified one fitted on 4 M walks). Fix: exhaustive testing of tiny-n instances of a parametric generator.
- **Free classical co-processor.** Classical bits are unpriced and unchecked at the end, and the current evaluator never checks that offset registers are unchanged (Joel §3.3).
- **Q is a static index, not live peak.**
- **Depth and reaction time.** Q × T ignores Toffoli depth and feed-forward latency. Already checked by Aodhfionn (C-AMP appendix): with a lazy Clifford-frame schedule, HEAD's reaction depth is 1.006× its Toffoli-only depth, so feedforward hides no meaningful time cost today.
- **The sub-900-qubit branch** is unpromoted and unaudited.

## Process lessons

- Identify circuits from metadata before building anything; never approve mass builds of contestant code.
- Keep agents in phases with stops; reports are only as good as their evidence labels.
- Wait for threads to settle before replying; keep drafts and positions in `replies/drafts/` so replies stay consistent.
