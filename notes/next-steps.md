# Next steps and candidate claims

## Follow-ups on this week's posts

1. **Strengthen F2 of the Fiat-Shamir reply (free patching).** `reply_v0.2.md` still says the zero-cost patch is "pending V1". V1 has since settled the mechanism: conditions on offset bits are free, and the condition stack ANDs them. A patch keyed on the test point's classical addend A_k therefore needs no Toffolis and no qubits:
   - invert the offset bits where A_k is 0 (free bit ops), push all 512 as conditions, apply X gates on the output bits where W_k ⊕ C_k = 1, then pop and restore;
   - it fires only on that one test shot, so patching costs ~0 T instead of ~46 T.

   This follows from V1 but has not been built. Two consequences:
   - **Limit of R1 in the classical-addend claim.** R1 as written does not block this patch, because it is an offset-conditioned X on qubits. Fresh secret inputs (M1) do block it.
   - **Next versions.** Worth a follow-up post, and a v0.3 of the reply and v0.2 of the claim.
2. **AT2/AT3 for the claim (sandboxed).** Build 54fc4f25. Count its R1 violations and k, and measure live qubits at the division seam against the peak. This settles the 6–31% range in the claim.
3. **Incumbent fresh-input failure rate.** Re-score 8e9c9a22 with only the 96 tail-X targets rewritten (fresh test points, same circuit). This verifies the ~0.19% figure both replies rely on, and needs no new contestant code.
4. **Ask the paper's authors** where the Supplemental Note F (windowed variant) code lives. It is not in the challenge repo. With it, the +384,708 split can be measured instead of modeled.
5. **Build T-F2** (the patch construction) on a toy simulator, to turn F2 from an argument into a demonstration.

## Open questions carried forward

- The leader's k (3–5) and seam width (0 to +256 qubits).
- Rank 1's bot-confirmed score (not fetched; rate limit).
- Whether a lower-Q promoted circuit exists among the 428 pre-August "Accept submission" commits, which carry no metrics.
- Unlookup cost to use in S_W: the literature's ~512 T, or a measured construction?
- Track G: K = 28 for all instances? ε_max option A (0.183%) or B (0.376%)? Can the runtime budget afford 30k fresh inputs?

## Candidate future claims

- **Support-calibrated approximation + scale-model verification.** Circuits are tuned until they pass the validation set (calibrated widths, census-mined gate stripping, an off-sample counterexample issue). Fix: build Track G's parametric generator at tiny n and test every input exhaustively, which catches structural bugs that sampling misses.
- **Free classical co-processor.** Classical bits are unpriced, unchecked at the end, and can compute arbitrary functions of measurement outcomes. Audit what the top circuits do with them beyond the addend.
- **Q is a static index, not live peak.** Q = highest qubit index + 1. Check whether allocation tricks make this diverge from true peak width in ways that matter.
- **Depth and reaction time.** Q × T ignores Toffoli depth and the classical feed-forward latency of measurement-based uncomputation. Low-qubit, high-T designs look better than they would run.
- **The sub-900-qubit branch.** It is unpromoted, has huge T, and was never audited for addend use or approximation.

## Process lessons

- Identify target circuits from metadata (`results.tsv`, PR bodies) before building anything.
- Never approve mass builds of contestant code. One sandboxed build per question, and only when source reading can't answer it.
- Keep agents in phases with a stop after each task. Their reports are only as good as the evidence labels.
