# #verified-claims catalogue (as of 2026-10-29)

This catalogue was read directly from Discord: every message in all six threads, plus inline previews of every attachment. Nothing was posted or downloaded. It records what is there; it makes no new claims.

**"In repo"** means the attachment is already in `context/teammate-claims/` or our own folders, with matching sizes. **"Captured"** means the text was taken from Discord's preview. Captures marked *partial* have gaps; download the original to replace them.

## Thread index

| # | Thread | Started by | Date | Messages | Our status |
|---|---|---|---|---|---|
| 1 | Eliminating Fiat-Shamir tail-nonce grinding (C-TAIL) | joel | Oct 16 | 10 | Replied Oct 19 (summary + follow-up + v0.2 doc) |
| 2 | Modulus Over Specialization (C-MOD) | Jude | Oct 18 | 8 | Replied Oct 19 (summary + follow-up + v0.2 doc) |
| 3 | Classical addend: restrict offset bits to loads | Elliot | Oct 19 | 4 | Our claim (v0.1 + audit). Two replies to it; not yet answered |
| 4 | Per-run error pricing (P003) | Mike | Oct 20 | 5 | **Replied Oct 20, 9:22 PM** |
| 5 | Error pricing undercharges approximate circuits by 2x in the exponent, and phase errors by 4x (C-AMP) | Aodhfionn Downs | Oct 22 | 3 | New; not answered |
| 6 | Predictable measurement records remain selectable after inputs are fixed (C-03) | Jack | Oct 22 | 3 | New; not answered |

## 1. C-TAIL (joel)

| When | Who | What | Attachment |
|---|---|---|---|
| Oct 16, 3:06 PM | joel | Original claim, three posts: problem statement, robust architecture, friction & impact | — |
| Oct 16, 3:14 PM | joel | "More detail" | `robust_architecture_specification.md` (22 KB), in repo |
| Oct 19, 7:56 PM (edited 10:14 PM) | Elliot | Reply summary; the follow-up correction about forward∘reverse is appended in the same message | — |
| Oct 19, 10:14 PM | Elliot | Detailed reply | `reply_fiat_shamir_v0.2.md` (15 KB), ours |
| Oct 20, 1:59–2:00 PM | Mike | R002 human summary, two posts | — |
| Oct 20, 2:00 PM | Mike | R002 agent doc | `R002_response_to_C-TAIL__AGENT.md` (12 KB), in repo |
| Oct 21, 4:05 PM | joel | "Improved version including synthesis of replies": two boards, five verification layers, provenance credits (C-TAIL, Elliot, Mike) | `shor-circuit-assurance-architecture.md` (82 KB), in repo |

Our posted summary still contains the original bullet "Pillar 3 already exists in the current harness". The correction is in the same message's follow-up paragraph.

## 2. C-MOD (Jude)

| When | Who | What | Attachment |
|---|---|---|---|
| Oct 18, 12:09 AM (edited 12:13 AM) | Jude | Original claim: problem statement and architecture solution (parametric generator, Track G, error-priced scoring) | — |
| Oct 18, 12:14 AM | Jude | "Machine readable document" | `ecdsafail_modulus_specialization_AGENT_SPEC_v0_2.md` (10 KB), in repo |
| Oct 19, 7:57 PM (edited 10:13 PM) | Elliot | Reply summary, plus follow-up (λ ≤ 18; `SUB4_LEGACY_POINT_ADD`) | — |
| Oct 19, 10:13 PM | Elliot | Detailed reply | `reply_trackg_v0.2.md` (13 KB), ours |
| Oct 20, 2:22 PM | Mike | R003 human summary, two posts | — |
| Oct 20, 2:22 PM | Mike | R003 agent doc | `R003_response_to_C-MOD__AGENT.md` (7 KB), in repo |
| Oct 21, 4:17 PM | joel | Track G synthesis summary | `reply_trackg_synthesis.md` (33 KB), in repo |

## 3. Classical addend: restrict offset bits to loads (Elliot)

| When | Who | What | Attachment |
|---|---|---|---|
| Oct 19, 10:07 PM | Elliot | Claim post | — |
| Oct 19, 10:08 PM | Elliot | Audit and claim doc | `audit_report.md` (86 KB) and `Claim — Classical-addend surface restrict offset bits to loads (v0.1).md` (17 KB), ours |
| Oct 22, 12:08 PM | joel | Track Q summary | `reply_trackq_synthesis.md` (41 KB), in repo |
| **Oct 23, 10:56 AM** | **Aodhfionn Downs** | **Reply measured on HEAD 72ccdb8** (below) | **`R_classical_addend_offset_bits__AGENT.md` (5 KB): captured in full** |

**Aodhfionn's reply (key content).**

- **Measurement.** Ops conditioned directly on offset bits (registers 2/3), via an offset-bit pass in `react_depth.rs`:

  | Circuit | Offset-bit writes | Quantum ops conditioned on an offset bit | Of those, Toffolis |
  |---|---|---|---|
  | baseline `fa473ea` | 0 | 0 | 0 |
  | July leader `60d6185` (8e9c9a2 lineage) | 0 | 0 | 0 |
  | HEAD `72ccdb8` | 0 | 5,120 | 0 |

- **Reading.** The diagnosis is correct in direction: the current leader conditions on offset bits where earlier ones did not. But all 5,120 ops are Cliffords or Paulis, so there is no Toffoli discount on this HEAD. This refines the claim's "claimed 7.5 T" to 0 T on his build. The structural point stands: those loads become qubit-controlled ops once A comes from a QROM table.
- **Refinement 1.** Phrase the rule as "an offset bit may appear as the condition of a load (X into a qubit) or of a classical bit op, and may not appear as the condition of a CCX/CCZ". Check the emitted op stream, not the source.
- **Refinement 2.** S_W "double-prices width". Tie the lookup term to the paper's measured windowed port (+11 qubits, +29.6% T), or to a measured QROM gadget.
- **Cross-thread.** An offset-keyed patch can zero the failure rate on a fixed public corpus at near-zero Toffoli cost, so both this fix and error pricing need a fresh or secret corpus.
- **Verification tasks.**
  - V1: the offset-bit pass at 72ccdb8 (expect 5,120 / 0 / 0; 0/0/0 on fa473ea and 60d6185).
  - V2: confirm the scorer ignores X/Z in Toffoli counts.
  - V3: build a 16-bit QROM gadget and measure it.

## 4. Per-run error pricing (Mike)

| When | Who | What | Attachment |
|---|---|---|---|
| Oct 20, 2:23 PM | Mike | P003 human summary, two posts | — |
| Oct 20, 2:23 PM | Mike | P003 agent doc | `P003_per_run_error_pricing__AGENT.md` (7 KB), in repo |
| **Oct 20, 9:22 PM** | **Elliot** | **Our reply:** "+1… Three suggestions": k = 28; a fixed public corpus undoes the penalty (offset-keyed patch ~0 T, inferred); one shared secret corpus per epoch; rounding and f = 0 nits | — |
| **Oct 23, 10:54 AM** | **Aodhfionn Downs** | **Reply: "right shape, wrong exponent"** (below) | **`R_P003_amplitude_exponent__AGENT.md` (6 KB): captured in full** |

**Aodhfionn's reply (key content).**

- **Agreement.** He accepts the rerun-cost frame and k = 28 (Google 2603.28846; ECDSA.Fail Eq. 11).
- **Correction.** A run is coherent, so success goes as the square of the surviving overlap:
  - wrong output or dirty ancilla: (1 − ε)^(2k);
  - phase-only: (1 − 2ε)^(2k).
- **Amendment.** Split failures per shot into W (wrong output), A (dirty ancilla) and P (phase only):
  - P_run = (1 − ε_W^UB − ε_A^UB)^(2k) · (1 − 2ε_P^UB)^(2k);
  - Score = round(T̄·Q / P_run);
  - 90% gate: ε_W + ε_A + 2ε_P ≤ 1 − 0.9^(1/56) ≈ 0.188%;
  - keep reporting the old number during transition.
- **Break-even.** A saving must beat 56·Δε (output/ancilla) or 112·Δε (phase), versus 28·Δε under P003.
- **Leader numbers.** HEAD 72ccdb8 = 743,575 T × 1,242 Q = 923.5M; 128,000 inputs independent of the op stream (`audit_eval.rs`):

  | Class | Count | Rate | 95% UB |
  |---|---|---|---|
  | Wrong output | 256 | 0.200% | 0.222% |
  | Dirty ancilla | 0 | 0 | — |
  | Phase only | 46 | 0.0359% | 0.0459% |

  | Pricing | P_run | Score |
  |---|---|---|
  | none | 1 | 923.5M |
  | P003 (1 − ε)^28 | 0.936 | 986.7M (+6.8%; 1 in 16 attacks fail) |
  | amplitude | 0.859 | 1,075.5M (+16.5%; 1 in 7 fail) |

  The leader passes the 0.376% gate (0.236%) and fails 0.188% (0.272%). Phase-only failures are 15% of the count but 26% of the penalty exponent.
- **Simulation.** `shor_error_toy.py`: closed form = brute-force state vector to 6 decimals at N = 11, and 1.000000 at ε = 0. Loss exponents at N = 101: wrong 1.5–1.8, ancilla 1.6–1.9, phase 3.0–3.7 (classical model predicts 1). The gap from 2 and 4 is attributed to a small-group artifact (`resid_vs_B.py`); a per-line model reproduces the phase case exactly.
- **Dirty resets.** A detected random-phase shot counted as class P prices dirty resets correctly. With Mike's deterministic flag, count them as class A; same expected cost.
- **Limits.** Assumes unstructured failures and P_ideal ≈ 1.
- **Verification tasks.** V1 `brute_check.py`; V2 `shor_error_toy.py 101 6` (~3.5 min); V3 `line_check.py`, `resid_vs_B.py`; V4 `audit_eval.rs` at 72ccdb8; V5 try to build an unstructured failure with r ≈ 1 at c ≥ 2^16.

## 5. C-AMP (Aodhfionn Downs), new thread

| When | Who | What | Attachment |
|---|---|---|---|
| Oct 22, 6:09 PM | Aodhfionn Downs | Problem / how we checked / leader audit | — |
| Oct 22, 6:09 PM | Aodhfionn Downs | Fix and sources (Gidney, *Approximate encoded permutations and piecewise quantum adders*; ECDSA.Fail paper; PostQuantum summary) | — |
| Oct 22, 6:10 PM | Aodhfionn Downs | C-AMP agent doc | `C-AMP_error_pricing_amplitude__AGENT.md` (17 KB): **captured, partial** |

The doc names its targets: P003, Elliot's reply to P003, joel's Track G synthesis, C-MOD F8, R002, and the paper's Q×T/p̂. Its content matches the P003 reply above, at more length:

- **Claims C1–C5.** C1 gives P_run ≈ P_ideal·(1 − ε_W − ε_A)^(2k)·(1 − 2ε_P)^(2k).
- **Derivation (A.1–A.4).** It positions Gidney's bounds (overlap ≥ 1 − 2ε, trace distance ≤ 2√ε) as consistent with C1 and looser.
- **Formula check** at N = 11, a sweep at N = 101, and a size sweep (N = 31 to 199): no size gives r near 1.
- **Leader audit.** Same numbers as the P003 reply: runs A/B 133/123 wrong and 23/23 phase; average executed T 743,567 / 743,570 against 743,575 official.
- **Requirements R1–R3.** R1 notes that the current `eval_circuit.rs` counts phase and ancilla failures only per 64-shot batch.

The doc says its scripts are attached, but only the `.md` is visible in the thread.

## 6. C-03 (Jack), new thread

| When | Who | What | Attachment |
|---|---|---|---|
| Oct 22, 10:49 PM | Jack | Claim post (problem, demonstrated primitive, unresolved, why it matters, fix) | — |
| Oct 22, 10:49 PM | Jack | "addtl info" | `C-03-measurement-outcome-grinding.md` (14 KB): **captured, partial** |
| Oct 23, 12:08 PM | Yara Abu Saude | Reply (below) | `Antigravity Technical Agent Report.txt` (7 KB): **captured, partial** |

**Claim.**
- **Mechanism.** The simulator continues the input XOF, and R/HMR consume 8 bytes each. Inserted clean resets shift later draws. Under a fixed public outcome sequence, padding can change whether a later dirty reset records a phase failure.
- **Demonstrated.** A two-qubit primitive with padding 0–15: padding 0 gave phase 0, padding 2 gave phase 1, with T = 0 and clean qubits. It does not establish a full invalid adder.
- **Unresolved.** The historical parent's 11.43 phase-failing shots per 9,024 are not attributed between changed inputs and changed outcomes.
- **Fix.**
  - Keep R/HMR rejected in the coherent profile.
  - In an optional measurement profile, a clean R consumes no outcome entropy and a dirty R rejects before erasure.
  - Outcomes come from an independent evaluator secret, indexed by logical trial, static HMR ordinal and repetition; padding must not advance a cursor.
  - Validate the corrected logical channel, with four outcome streams (one primary, three diagnostic).

**C-03 doc.** Finding C-03, revision 2.1, P0, distribution private, acceptance tests NOT_RUN. It has nine sections (summary, scope, architecture, invariants, reference algorithm, fixtures, operational impact, checklist, evidence limits). Notable content:
- an outcome bit derived by HMAC-SHA256 over a per-repetition seed, logical input index and HMR ordinal;
- a reset contract: q & live_mask & active_condition must be 0 before R;
- the corrected-branch condition K_m = c_m(O)·U_O;
- an evidence-limits note: survival is (1 − d)^m, and 2^−m is an upper bound only if d ≥ ½;
- the inspected sources match historical pin `0ae6d109…`.

**Yara's reply.**
- **Agrees with:** the mechanism is real.
- **Disputes:** that it is an open exploit in the actual harness. The Fiat-Shamir seed hashes the whole op stream, so padding reshuffles both inputs and draws; and evading detection across 9,024 shots is statistically implausible.
- **On the fix:** clean-R-zero-entropy plus dirty-R-reject "completely closes" the loophole; the indexed secret and multi-stream checker are "over-engineered".

**Antigravity report.**
- verifies the XOF continuation, clean-reset and dirty-reset behaviour with citations (`eval_circuit.rs:204-218, 248-254, 273`; `sim.rs:140-155`);
- reproduces the primitive with a fixed seed;
- calls the Fiat-Shamir binding a fatal gap in the claim;
- rates "dirty R hard-rejecting before erasure" the optimal fix;
- flags the 11.43 figure as not verifiable in its repo.

## Facts that cut across threads

- **Leaderboard moved.** 7499c90 (933,231,010 = 756,265 × 1,234; P003/R002), then **72ccdb8 (923.5M = 743,575 × 1,242; Aodhfionn, Oct 22–23)**.
- **Leader error rates on independent inputs.** 7499c90: 0.208% any-channel (Mike). 72ccdb8: 0.200% wrong + 0.036% phase-only, no dirty ancilla (Aodhfionn).
- **k = 28** is now used by Elliot, Aodhfionn and joel's assurance doc. Mike's P003/R003 and joel's Track G synthesis use 32.
- **Offset-keyed patching of a fixed public corpus** has been raised independently by Elliot (P003 reply; Fiat-Shamir reply F2) and Aodhfionn (classical-addend reply).
- **The error-pricing exponent is now contested.** It is (1 − ε)^k in P003, our reply and Track G, and (1 − ε)^(2k) / (1 − 2ε)^(2k) in Aodhfionn's C-AMP.
- **Measurement and reset handling** has three proposals: deterministic dirty-R flagging (Mike, R002); dirty-R reject plus an independent indexed outcome secret (Jack, C-03); and class-P pricing that covers dirty resets (Aodhfionn).
- **Not yet answered by us:** Track Q (thread 3), Aodhfionn's two replies (threads 3 and 4), C-AMP (5), C-03 (6), and R002, R003, joel's two syntheses (threads 1 and 2).
