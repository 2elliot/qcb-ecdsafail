# Claim — Classical-addend surface: restrict offset bits to loads (v0.1)

2026-10-19 · Elliot D'Alessandro

## TL;DR and status

The harness supplies the addend A as classical bits, which lets circuits condition gates on it for free. Windowed Shor, the interface real attack estimates use, cannot do this, because there A is quantum data loaded by a table lookup.

- **Older leaders:** none of the three measured circuits used the surface (0 addend-conditioned Toffolis).
- **Current leaders:** the top lineage has started using it.
- **Fix:** offset bits may only drive X-loads into qubits, and the leaderboard reports a windowed-adjusted score alongside S.

| Status | What |
| --- | --- |
| Measured | V1 mechanism (harness at commit `3dee0cd6`); V2 exposure of 8e9c9a22, 3616dbf2 and 3493c837 with the official scorer reproducing the published Q and T exactly; the 3·x_A replacement cost (+433.4 T) |
| Source-read, not built | The top-3 lineage's offset-conditioned first carries and classical-factor passes |
| Claimed by submitters | Ranks 1–3 Q and T, rank 1's S, the passes' −6.5 / −1.0 T savings |
| Inferred | Number of addend-access windows k for the top lineage; width cost of porting it; the x_A + 1 replacement cost |

DRAFT v0.1. The full audit trail is `audit/audit_report.md`, produced by Claude Code under a written brief (`audit/agent_brief.md`).

## The mechanism

The harness at `3dee0cd6` prices three things for free that windowed Shor must pay for: gates conditioned on addend bits, classical logic on those bits, and loading A into qubits. All file:line references are to that commit.

| Fact | Evidence |
| --- | --- |
| Registers 2 and 3 (offset_x, offset_y) must be classical bits, loaded per shot | `eval_circuit.rs:471-508`, `286-293` |
| Every op can carry a classical condition, via the `c_condition` field or the PushCondition/PopCondition stack | `circuit.rs:104-113`; `sim.rs:73-80`, `168-176` |
| A CCX/CCZ counts only on shots where its condition is true (`popcount(cond)`); T is the average over shots | `sim.rs:82-87`; `eval_circuit.rs:357-361` |
| X, Z and the classical bit ops (BitInvert, BitStore0/1) are uncounted; with the condition stack they give free classical AND/XOR | `sim.rs:95-98`, `156-164` |
| Classical bits are not in Q (highest qubit index + 1) and are never checked at the end | `circuit.rs:355-363`; `eval_circuit.rs:326-340` |
| The test filter removes A = O, R = O and R = ±A, so no circuit is ever tested on an identity row | `eval_circuit.rs:257-265` |

Two consequences follow:

- **Free conditioning.** A Toffoli with an addend-bit control can become `CX … if ox[i]`: zero Toffolis.
- **Free derived constants.** Values like 3·x_A mod p can be computed at run time with uncounted bit ops. They cannot be computed at build time, because the offsets are drawn from a hash of the finished op stream.

In windowed Shor (w = 16, 28 additions per attack), A comes out of a quantum table lookup, so neither applies. Derived constants are the exception: they can be stored as extra table columns for free, since a lookup's Toffoli cost does not depend on payload width.

## Measured: the earlier leaders do not use it

Three circuits were built and run with the official scorer. None has a single Toffoli whose condition depends on the addend. The scorer reproduced each circuit's published Q and T exactly, and the counting instrument matched the real simulator on every batch.

| Circuit (commit) | Role | Q | T | Offset-conditioned CCX/CCZ | Offset-dependent executed T/shot | T in addend windows |
| --- | --- | --- | --- | --- | --- | --- |
| 8e9c9a22 (`60d61859`) | Paper's incumbent | 1,151 | 1,299,452.7 | 0 of 1,358,316 | 0.000 | 1,600.1 |
| 3616dbf2 (`897dda2b`) | Ping-pong GCD | 1,321 | 952,707.5 | 0 of 1,009,077 | 0.000 | 1,600.0 |
| 3493c837 (`31f9c58c`) | Lowest-Q promoted (with PR metrics) | 1,150 | 1,284,776.1 | 0 of 1,340,596 | 0.000 | 1,592.6 |

- **Where A enters.** All three share one coordinate shell. A enters at five modular additions or subtractions, each about 320 T and each already running on a 256-qubit temp register. It never enters the inversion, squaring or multiplication.
- **Which ops see the offsets.** The only quantum op ever executed under an offset condition is X: 2,560 = 10 load/unload runs × 256 bits. The rest is classical bookkeeping.
- **Free derived constants.** Two constants are computed with free classical bit ops: 3·x_A mod p and x_A + 1.
- **Bound on the discount.** Computing 3·x_A in-circuit from quantum x_A costs +433.4 T (measured: 755.0 vs 321.5 T on the same 9,024 inputs). Replacing x_A + 1 costs at most about 0.32k T (inferred). The total arithmetic-level discount is therefore at most about 0.75k T, under 0.06% of T.

## The paper's windowed overhead is lookup cost

The ECDSA.Fail paper reports a windowed-compatible variant of 8e9c9a22 at +11 qubits and +384,708 Toffolis (+29.6%). That overhead is table-lookup work, not lost classical specialization.

- **Size in lookups.** One unary-iteration lookup at w = 16 with measurement-based spine uncompute costs 65,534 T (= 2^16 − 2). This was counted on the PR #15 construction, which is validated exhaustively for w ≤ 6. So +384,708 is about 5.87 lookup traversals.
- **What the paper describes.** Three stages that load A just before use, each cleared by exact reversible replay of its lookup: 6 traversals ≈ 393,204 T. This is consistent with the total, but the model was picked from a small set to fit it, so treat it as consistency, not evidence.
- **Why the specialization share is small.** The arithmetic that touches A is about 1,600 T and already runs on qubits (previous section). It cannot hide 188k Toffolis.
- **Code not available.** The variant's code is not in the challenge repo. The audit searched 1,120 branch heads, 14,152 reachable blobs and 803 PR heads. Its numbers are the paper's, not ours.

**Implication for accounting.** The standard analytic charge is 3·2^16 Toffolis (three lookups, cheap unlookups) plus 16 address qubits. A 3-stage port that clears its payloads with measurement-based unlookup (about 2·2^8 Toffolis each) should land close to that. The paper's extra ~188k reflects its choice of replay unloads, so it is not evidence that the analytic charge under-prices these circuits.

## Emerging: the current top lineage conditions arithmetic on offset bits

The current top three do what the older leaders did not: they condition arithmetic gates directly on offset bits. This was read from source at their commits; their op streams were not built.

| Rank | Submission (commit) | S | Q | T |
| --- | --- | --- | --- | --- |
| 1 | 54fc4f25 (`3dee0cd6`, PR #817) | 939,138,168 (claim) | 1,234 (claim) | 761,052 (claim) |
| 2 | d9ae2845 (`90e777ea`, PR #814) | 939,169,018 (bot-recorded) | 1,234 (claim) | 761,077 (S/Q) |
| 3 | 9dc3798c (`0c3a0751`, PR #718) | 939,192,464 (bot-recorded) | 1,234 (claim) | 761,096 (S/Q) |

The lineage uses the surface in three ways:

1. **Offset-conditioned first carries, by construction, in all three ranks.**
    - The x-side adder `r5_ripple_add_cbits` computes its first carry as `push_condition(ox[0]); cx(...)` (`modular.rs:1004-1005`).
    - Later carries are CCX gates with offset-conditioned X flips.
    - The saving is about 3 T/shot, the same across the three ranks.
2. **Classical-factor passes, in ranks 1–2 only** (`classical_factors_xor.rs`, `classical_wide/`; called at `mod.rs:620, 622`).
    - They rewrite any CCX whose control provably equals a classical bit into a conditioned CX, and offset bits are valid witnesses.
    - The claimed saving is −6.5 and −1.0 T (PR #817).
3. **No qubit register for the addend on the x side.** The x-side steps take the offsets as classical operands, seamed into the division seed and unseed (`leapfrog.rs:3661-3666`, `6326-6342`), with no 256-qubit temp register. A windowed port must hold A in qubits there.

**Ranking impact.** The Toffoli effect cannot flip the top three: adjacent S gaps are 30,850 and 23,446, while charging back all 7.5 claimed Toffolis costs at most 1,234 × 7.5 = 9,255. The width effect is the open question. If the division seam sits near the circuit's peak, holding A there could cost up to +256 qubits, about +21% of Q. This is inferred, not measured.

## What the interface is worth

The leaderboard shows S falling 37.2% from the incumbent to today's leader. Under windowed accounting the same improvement is between 6% and 31%, depending on two unmeasured properties of the leader. The gap exists because lookup cost is a fixed per-addition charge, so lower-T circuits pay a larger share of it.

Windowed-adjusted score:

```math
S_W = (Q + \Delta Q)\,\bigl(T + k\,(2^{16}-2) + k \cdot 512\bigr)
```

Here k is the number of addend-access windows, ΔQ is 16 address qubits plus any width needed to hold A, and 512 Toffolis is an inferred measurement-based unlookup cost. The incumbent has k = 3: its five load sites form three windows with zero Toffolis between adjacent pairs.

| Leader scenario (54fc4f25) | Windowed S | Drop from incumbent (S_W = 1.748 × 10^9) | Lookup share of T |
| --- | --- | --- | --- |
| Leaderboard as published (no adjustment) | 0.939 × 10^9 | 37.2% | — |
| k = 3, no width cost | 1.199 × 10^9 | 31.4% | 20.7% |
| k = 5, no width cost | 1.364 × 10^9 | 21.9% | 30.3% |
| k = 3, +256 qubits at the division seam | 1.445 × 10^9 | 17.3% | 20.7% |
| k = 5, +256 qubits at the division seam | 1.643 × 10^9 | 6.0% | 30.3% |

- **What is known:** the leader's k is between 3 and 5, since it has five addend-access windows whose merging is unknown.
- **What is not:** whether the x-side seam needs extra width. Both depend on the op stream.
- **What the table shows:** even the best case means the leaderboard overstates progress by about 6 percentage points. The worst case would erase most of it. Measuring k and the seam's live width (open question 1) settles which.

All leader inputs are submitter claims (Q 1,234, T 761,052).

## Proposed fix

The key rule is R1: offset bits may only drive X-loads into qubits. Under it, any use of A requires A in qubits, so the width cost of holding A shows up in Q automatically. The remaining windowed difference (lookup traversals and address qubits) is then charged analytically. MUST/SHOULD are per RFC 2119.

| ID | Requirement |
| --- | --- |
| R1 (MUST) | A classical condition whose taint includes an offset bit may appear only on (a) X gates targeting qubits (loads and unloads), and (b) classical bit ops whose outputs are themselves used only as conditions on such X gates (derived constants such as 3·x_A mod p and x_A + 1, which become extra table columns in windowed Shor). Anything else conditioned on offset-derived bits (CX, CZ, CCX, CCZ, Swap, R, HMR, Neg) is rejected with `ADDEND_CONDITION_VIOLATION` |
| R2 (MUST) | A condition whose taint mixes offset bits and measurement outcomes is rejected. A function of both A and a measurement result has no classical-table equivalent when A is quantum |
| R3 (MUST) | The scorer counts addend-access windows k statically. A window opens at the first offset-conditioned X load and closes when every loaded qubit has been unloaded; windows separated by zero executed Toffolis merge |
| R4 (SHOULD) | The leaderboard reports S_W = (Q + 16)(T + k(2^16 − 2) + 512k) next to S |
| R5 (SHOULD) | The check runs as a static taint pass over `ops.bin`, before simulation. Our `qcb_audit/taint_count --static` is a reference implementation |
| R6 (SHOULD) | Track S rankings are unchanged; S_W is reported, and adopting it for ranking is a separate decision |

**Effect on today's submissions:**

- **8e9c9a22, 3616dbf2 and 3493c837 pass R1 and R2 unchanged.** Their only offset-conditioned quantum op is X.
- **All three current top-ranked circuits are expected to fail R1**, because of the offset-conditioned first carries. Ranks 1–2 also fail through their classical-factor rewrites. Rework means loading A into qubits at those sites: about 10 T, plus whatever width the division seam needs.

The rule composes with the other proposals:

- **Track G** (`build(&ModulusParams)`): it adds no generator inputs.
- **Fresh-input design** (Fiat-Shamir reply): it is a static check and needs no test inputs.

## Acceptance tests and verification tasks

AT1–AT3 confirm the evidence above; AT4–AT6 test the fix itself. AT2 and AT3 share the only build of untrusted contestant code.

| ID | Test | Expected |
| --- | --- | --- |
| AT1 | Static taint pass on 8e9c9a22, 3616dbf2 and 3493c837 `ops.bin` | R1/R2 pass; offset-conditioned quantum ops are X only (2,560); k = 3 for each |
| AT2 | Build 54fc4f25 and run the same pass | R1 violations at the x-side first carries and the classical-factor rewrites; report their count, executed T/shot, and k |
| AT3 | On 54fc4f25, compute live qubits (first-to-last use per index) during each addend window vs the global peak | Settles the width question: is the division seam within 256 qubits of the peak? |
| AT4 | Synthetic streams: (i) only offset-conditioned X loads; (ii) one offset-conditioned CX; (iii) one condition mixing offset and HMR bits | (i) accept; (ii) reject R1; (iii) reject R2 |
| AT5 | Synthetic windows: two loads separated by 0 Toffolis; two loads separated by 1 Toffoli | k = 1; k = 2 |
| AT6 | Runtime of the static pass on a ~9 M-op stream | Well under the scorer's own runtime; the full static + dynamic instrument already took 37–72 s |

AT1 re-runs work already done in the audit. AT4 mirrors the instrument's existing positive control: a synthetic stream with two offset-conditioned CX and two CCX reported the expected 4.993 T/shot.

**Safety:** AT2 and AT3 run untrusted contestant code. Do it in a sandbox (WSL2, a container or a VM), not on a personal machine.

## Open questions

1. **Width at the division seam.** Does holding A in qubits at the leader's x-side windows raise its peak? This is the largest open term: 0 to +256 qubits (AT3).
2. **The leader's k.** It is between 3 and 5, from five windows whose merging is unknown (AT2).
3. **The paper's variant code** (Supplemental Note F) is not in the challenge repo. Ask the authors; with it, the +384,708 split can be measured instead of modeled.
4. **Unconfirmed leaderboard numbers.** Rank 1's bot score was not fetched (API rate limit). The 428 pre-August "Accept submission" commits carry no metrics, so a lower-Q promoted circuit among them cannot be ruled out.
5. **Unlookup cost.** The 512-Toffoli measurement-based unlookup is a literature estimate, not built here. Should R4 use it, or a measured construction?
6. **Unpriced costs.** Payload CNOTs (Clifford) and lookup ancillas at the peak are unpriced, as in the paper's analytic proxy.
7. **Double-and-add is out of scope.** There the addend is legitimately classical, but every addition needs a control qubit and there are 512 calls instead of 28, so it is not the attack this kernel is used in.

## Reproduction and sources

**Reproduction.**
- **Harness.** Layr-Labs/ecdsafail-challenge at `3dee0cd6`. The harness files are unchanged since `c66c9586` (2026-06-22), so V1 applies to every audited commit.
- **Instrument.** `qcb_audit/taint_count/` is a standalone crate on the harness library, with no new dependencies. Its static pass taints classical bits as offset- or measurement-derived; its dynamic pass replays the official inputs and cross-checks every batch against `quantum_ecc::sim::Simulator`.
- **Raw outputs.** `qcb_audit/results/`.
- **Build setup.** Builds used stable rustc 1.99.1, `--locked --offline`, outside the repo; the official scorer reproduced every published Q and T exactly.

**Sources.**
- QCB audit report, `audit/audit_report.md`: V1–V4, the Q1–Q4 follow-up, and the critique pass
- [ecdsafail-challenge repository](https://github.com/Layr-Labs/ecdsafail-challenge) and its [issue list](https://github.com/Layr-Labs/ecdsafail-challenge/issues)
- [ECDSA.Fail paper, arXiv:2609.09582](https://arxiv.org/pdf/2609.09582), §5.2.3 and Supplemental Note F: windowed-compatible variant, +11 qubits, +29.6% T
- [Babbush et al., arXiv:2603.28846](https://arxiv.org/html/2603.28846v2): w = 16, 28 windowed additions, measurement-based uncomputation of lookups
- [Schrottenloher, arXiv:2606.02235](https://arxiv.org/html/2606.02235v1): per-addition lookup accounting of +16 qubits and 3·2^w Toffolis
- [Trapped-ion ECDLP estimate, arXiv:2609.05625](https://arxiv.org/html/2609.05625): random offset masks; union bound over 28 additions
- Challenge PRs #15 (ADR 0010/0013/0015), #718, #814 and #817: lookup cost construction, addend width study, top-3 submissions
