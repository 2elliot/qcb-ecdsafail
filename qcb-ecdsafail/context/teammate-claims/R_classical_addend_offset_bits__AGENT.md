<!-- Captured 2026-10-29 from the Discord inline preview (Aodhfionn Downs, #verified-claims, thread "Classical addend: restrict offset bits to loads", Oct 23). Text complete; markdown formatting reconstructed. Replace with the original attachment if downloaded. -->

# Reply to the classical-addend surface claim — measured on HEAD, diagnosis holds, the Toffoli discount is not there yet

Self-contained. Reply to the claim "The classical-addend surface ... restrict offset bits to loads" (proposes offset bits may only drive X-loads, and reporting S_W = (Q + 16)(T + k·(2^16 − 2) + 512k)).

**Type:** reply with a direct measurement and one amendment. **Status:** measured on HEAD; the windowing argument is read, not rebuilt. **Repo measured:** ecdsafail/ecdsafail-challenge at 72ccdb8 (HEAD, 2026-10-23), stable Rust 1.97.

## What I measured

I instrumented the op stream (react_depth.rs, offset-bit pass) to count, over the four declared registers, writes to offset bits and quantum ops whose condition is a register-2 or register-3 (offset) bit:

| circuit | offset-bit writes | quantum ops conditioned directly on an offset bit | of those, Toffolis |
|---|---|---|---|
| baseline fa473ea | 0 | 0 | 0 |
| July leader 60d6185 (8e9c9a2 lineage) | 0 | 0 | 0 |
| HEAD 72ccdb8 | 0 | 5,120 | 0 |

So the diagnosis is correct in direction. The current leader does condition arithmetic directly on offset bits, where the earlier leaders did not (0 to 5,120). But on this HEAD all 5,120 are Cliffords and Paulis, and zero are Toffolis. X and Z are not even counted by the scorer (sim.rs tracks them classically), and CX/CZ are Clifford, outside the Q×T score. The offset-conditioned work on HEAD therefore buys no Toffoli discount. This matches the claim's own audit note that the current top-3 "classical-factor" passes are too small to flip the ranking, and refines its "claimed 7.5 T" to 0 T on the HEAD I built.

The structural point still stands. Those 5,120 offset-conditioned loads are free here only because A arrives as classical bits. In a windowed port A is a QROM output in qubits, so each of them becomes a qubit-controlled op. That is real cost, and it is exactly the surface the claim wants to close.

## On the proposed fix

Restricting offset bits to X-loads (plus classical bit ops for derived constants, folded into table columns) is a clean static rule and a taint checker can enforce it in one pass. Two refinements:

**Rejection is not free to specify.** The first carries the thread calls "by construction" are the ones that show up as the 5,120 conditioned Cliffords. A rule that rejects "anything else conditioned on an offset bit" has to allow those, or legitimate mixed-addition carry logic gets rejected. Suggest phrasing the rule as: an offset bit may appear as the condition of a load (X into a qubit) or of a classical bit op, and may not appear as the condition of a CCX/CCZ. That bans the only form that could hide a Toffoli discount and leaves the Clifford carries alone. The checker then runs on the emitted op stream, not the source.

**The S_W formula double-prices width.** S_W = (Q + 16)(T + k·(2^16 − 2) + 512k) adds the lookup Toffolis and a +16 qubit term and a +512k width term, multiplied out. The paper's own windowed port measured the whole coherent cost at +11 qubits and +29.6% Toffolis over the mixed incumbent (1,684,161 vs 1,299,453), which is the lookup plus unlookup already. Reporting S_W next to S is useful, but tie the lookup term to the measured +29.6% rather than a fresh analytic budget, or the two numbers will disagree with the paper and with each other. If the goal is a single generalization-honest score, the per-window lookup should be measured on the actual QROM gadget, not assumed.

## Relation to error pricing

The offset surface and the error-pricing thread interact. A patch keyed to offset bits can push the failure rate to zero on a fixed public corpus at near-zero Toffoli cost, since offset-bit conditions are free here. That only works if the scoring corpus is fixed and visible. It is the same reason the error-measurement corpus has to be fresh or secret (C-TAIL, Track G). Worth stating in both threads so the fix is not undone by the other.

## Verification tasks

- **V1** Build at 72ccdb8. Add the offset-bit pass to an op-stream reader (it reads the 4 registers from analyze_ops, marks register-2 and register-3 bits, and counts ops whose c_condition is a marked bit). Expect 5,120 conditioned ops, 0 Toffolis, 0 offset-bit writes on HEAD; 0/0/0 on fa473ea and 60d6185.
- **V2** Confirm the scorer ignores X/Z in the Toffoli count (sim.rs, the comment at OperationType::X) so that offset-conditioned Cliffords cannot appear in S regardless.
- **V3** Build the windowed QROM gadget for one 16-bit window and measure its Toffoli and width cost directly, to replace the analytic k·(2^16 − 2) + 512k term in S_W.
