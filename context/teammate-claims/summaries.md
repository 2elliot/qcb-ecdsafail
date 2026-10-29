# Teammates' human summaries (as posted in Discord)

Kept so the agent docs next to this file have their human-facing posts alongside. Lightly reformatted; content unchanged.

## P003: Pricing errors per full attack (per-run error pricing)

Agent doc: `P003_per_run_error_pricing__AGENT.md`.

- **Idea.** The leading circuit fails on ~0.21% of inputs (wrong answer, uncleaned scratch qubit, or leftover phase). A Shor attack chains many point additions and fails if any one goes wrong. Failed attacks are detectable (check the recovered key against the public key) and rerun, so true cost = run cost ÷ P(run succeeds). Score = Toffoli × qubits ÷ (1 − ε)^k. A less reliable circuit scores worse; an exact one is unaffected. First-order model: assumes random-looking inputs for every addition.
- **k.** Litinski (2023): 16 windowed additions per register × 2 registers = 32 per run (13 in a brute-force variant). Neither paper states "32" verbatim. Google 2026 allows the adder to be wrong on a small fraction of inputs. Recommends k = 32 (range 13–32), plus a k = 1 score for comparison.
- **Effect on leader (measured).** None: 933.2 M; k = 1: 935.4 M (+0.2%); k = 32: 1,005.6 M (+7.8%), one attack in 16 fails. Median genuine improvement ~0.016%. Leader passes a max-error gate of ~0.33%.
- **Catch.** A heavier penalty magnifies measurement luck. Measuring 0.21% precisely at k = 32 needs billions of inputs. Fixes: test more inputs only for circuits that err; a max allowed error (~0.33%); expect error rates to fall (≈50 min on a laptop at 0.001%).

## Track Q synthesis (response to my classical-addend claim)

Agent doc: `reply_trackq_synthesis.md`.

- The claim caught the classical-addend loophole, but its static taint rules and S_W formula were structurally flawed; the "consensus reply" replaces them with Track Q, proposed as the new default.
- **Why taint failed.** It is syntactic, not semantic, and breaks on reversible ancilla uncomputation. Idle-anchor flaw: an idle gate holds a window open, reducing k and corrupting S_W.
- **Track Q.** All four interface registers (R_x, R_y, A_x, A_y) are 256 distinct qubits; aliased wires rejected. Circuits may borrow or recode A but must restore it. Profile A: Q_A = max(Q + 16, 1055), T_A = T + 131,068 (16-qubit QROM load + replay).
- **Legacy dropped.** Top legacy circuits relied on classical carry conditioning, TAIL_NONCE grinding and free derived constants; Track Q "reveals the true physical cost" (S_A ≥ 1.28 × 10⁹, 36–75% above legacy).
- **Friction.** Runs by default in benchmark.sh and CI; under 2 minutes and 1.1 GB on free GitHub runners; legacy via `--legacy`.

## R003 (Mike): response to the modulus-specialization claim (C-MOD)

Agent doc: `R003_response_to_C-MOD__AGENT.md`.

- **Checked.** The special-prime shortcut is hardcoded in six source files. Internal sizes are fitted to sampled inputs with a deliberate failure budget. C-MOD's ~0.19% failure matches Mike's 0.21% (all failure types; 0.17% wrong answers only) on the current leader.
- **Why it deviates from the task.**
  1. Using secp256k1's special prime is real optimization; an attacker would use it.
  2. It leaves grinding untouched on the current leaderboard: every accepted submission since Oct 8 changed the nonce; one accepted "improvement" changed only the nonce.
  3. It breaks offline-exact scoring: scores depend on secret primes revealed after the contest closes.
  4. Its error penalty is too weak: it prices one addition; an attack runs ~32. At the 0.5% ceiling ~15% of attacks fail while the score rises 0.5%.
  5. Bigger change: generators instead of circuits, ~10 builds each, off-curve test points on other primes.
- **Keep.** Inputs independent of the circuit with separate gate-count and error sets; any failure type counted; conservative error estimate; max allowed error. Track G can stay an optional "generality" research track.

## Joel: Track G synthesis specification

Agent doc: `reply_trackg_synthesis.md`.

- Synthesizes C-MOD v0.2, Elliot's reply (v0.2) and Mike's R003.
- **From Elliot.** Compounded error pricing ((1 − ε₀)/(1 − ε_UB))^K; randomized fresh corner testing (fixed corners patchable for ~46 T); generator purity (OS sandbox, zero ambient environment).
- **From Mike.** K = 32 (Litinski 2023, Google 2026); legitimacy of specialization; Track S kept bit-identical.
- **Synthesis.** Soft floor T_eff = max(T_secp, G_siblings/1.50), capping specialization gains at 9.6%; Layer 3 sampling decoupled from Layer 4 pricing; N = 10,000 and ε_max = 0.0075 (eliminates v0.4's 59.5% stochastic rejection); decoupled PRNG sub-sponges for Miller-Rabin bases.

## R002 (Mike): response to the tail-nonce claim (C-TAIL)

Agent doc: `R002_response_to_C-TAIL__AGENT.md`.

- **Measured.** The leader is wrong on 0.17% of inputs (~16 per 9,024); a random nonce passes ~1 in 6 million tries. All 37 accepted submissions since Oct 8 changed the nonce. One accepted "improvement" changed only the nonce (identical circuits; the 14-T gain was luck). Official Toffoli average swings ±7 by nonce; 18 of 88 October improvements are within that range. Median genuine improvement 0.016%.
- **What's wrong with C-TAIL.** The gate filter catches one trick (renaming, reordering, spacing also change the hash). Reverse execution can't work with measurements, and the README claims a check that doesn't run. Fixed public points are right, but 10,000 points with zero failures is still gameable, and the safety argument is wrong. The 15% figure is ~100× too high.
- **Proposed (PFS-priced).** Requirements (identical submissions treated the same at any time; exact score computable offline) rule out secret salts, so make luck worthless, not impossible: published seed independent of the circuit; average Toffoli on a fixed input set; error rate on a large fixed set, priced into the score; dirty resets always counted as failures.
- **Limits.** One million inputs (~3 min) isn't enough to make luck smaller than today's steps for approximate circuits; test more inputs only for circuits that err. Tuning to public points is still only a worst-case estimate; rotating the seed removes it.

## Joel: Shor circuit assurance architecture (synthesis of C-TAIL replies)

Agent doc: `shor-circuit-assurance-architecture.md`.

- Sampled classical tests catch bugs but cannot prove quantum coherence.
- **Two boards.** A research board for sampled development scores; a qualified board for circuits with proven error bounds inside a Shor wrapper profile.
- **Five verification layers.** Structural checks; SIMD testing with one failure mask (wrong coordinates, dirty ancillae, phase, dirty resets); stress tests incl. R = A, −A, −2A; coherent state-vector simulation of toy curves; compositional proof certificates.
- **Provenance.** From C-TAIL: nonce diagnosis; peephole filter and reverse pass removed. From Elliot: patching argument; fresh challenges, R = −2A, no complete-adder mandate. From Mike: 0.173% error, dirty resets, Clopper-Pearson. Freezes circuit digests before public randomness beacons; evaluates error across superpositions.

---

# Added 2026-10-29 (from the #verified-claims catalogue)

## Aodhfionn Downs: reply to the classical-addend claim (Oct 23)

Agent doc: `R_classical_addend_offset_bits__AGENT.md`.

- **Measured.** On HEAD 72ccdb8, 5,120 ops are conditioned directly on offset bits. The baseline and the July leader have none. All 5,120 are Cliffords or Paulis and 0 are Toffolis, so there is no Toffoli discount today; this refines the claimed 7.5 T to 0 T.
- **Structural worry still stands.** Those free loads become qubit-controlled ops once A comes from a QROM table.
- **Refinements.** Phrase the rule as "an offset bit may condition a load or a classical bit op but never a CCX/CCZ", checked on the emitted stream. S_W double-prices width; tie the lookup term to the measured gadget or the paper's +11 qubits / +29.6% T.
- **Cross-thread.** An offset-keyed patch can zero the failure rate on a fixed public corpus at near-zero Toffoli cost, so this fix and error pricing both need a fresh or secret corpus.

## Aodhfionn Downs: reply to P003 (Oct 23)

Agent doc: `R_P003_amplitude_exponent__AGENT.md`.

- **Agrees with:** P003's structure and k = 28.
- **Correction.** A coherent run's success goes as the square of the surviving overlap: (1 − ε)^(2k) for wrong output or dirty ancilla, and (1 − 2ε)^(2k) for phase-only errors. This is checked by exact simulation (loss exponents 1.5–1.9 and 3.0–3.7 at N = 101, where the classical model predicts 1).
- **Leader.** HEAD 72ccdb8 (923.5M): 0.200% wrong, 0.036% phase only, no dirty ancillas, on 128k independent inputs. P003 prices this at +6.8% (1 in 16 attacks fail); the amplitude version at +16.5% (1 in 7). The 90% gate tightens from 0.376% to ε_W + ε_A + 2ε_P ≤ 0.188%, which the leader fails.
- **Amendment.** Classify failures per shot into three classes; Score = T̄·Q / [(1−ε_W−ε_A)^(2k)·(1−2ε_P)^(2k)]; keep reporting the old number during transition.

## Aodhfionn Downs: C-AMP, "Error pricing undercharges approximate circuits by 2x in the exponent, and phase errors by 4x" (new thread, Oct 22)

Agent doc: `C-AMP_error_pricing_amplitude__AGENT.partial.md` (partial capture).

- **Problem.** P003, Elliot's reply, the Track G synthesis and the paper's Q×T/p̂ all price errors as independent coins. A coherent run gives (1 − ε)^(2k) for wrong or ancilla errors and (1 − 2ε)^(2k) for phase-only errors. The coin model overstates success, so it rewards adding error.
- **Checked by exact simulation** of windowed discrete-log phase estimation (one million paths). The amplitude model reproduces phase errors exactly; the remaining gap for the other two shrinks as each failing input covers a smaller share of paths (≤ 2⁻³² in real secp256k1).
- **Leader (72ccdb8).** 0.200% wrong and 0.036% phase-only. k = 28 pricing: +6.8% under P003 vs +16.5% under the amplitude model. The 90% gate drops to 0.188%; the leader sits at 0.272%.
- **Fix.** Classify every shot as wrong output, dirty ancilla or phase only (the scorer currently reports phase and ancilla per 64-shot batch). Score = T̄·Q / P_run with P_run = (1 − ε_W − ε_A)^56 · (1 − 2ε_P)^56, using upper confidence bounds. A saving must beat 56× the added wrong-output rate or 112× the added phase rate, versus 28× under P003.

## Jack: C-03, "Predictable measurement records remain selectable after inputs are fixed" (new thread, Oct 22)

Agent doc: `C-03-measurement-outcome-grinding.partial.md` (partial capture).

- **Problem.** The historical simulator continues the input XOF; each R or Hmr consumes eight bytes. An inserted reset of an already-zero scratch qubit shifts later draws, so with a fixed public outcome sequence, padding can change whether a later dirty reset records a phase failure.
- **Demonstrated.** A two-qubit steering primitive: with one fixed seed, 0–15 inserted clean resets changed the final phase (padding 0 → phase 0, padding 2 → phase 1), with T = 0 and clean qubits. Not shown for a full adder or quantified for a frontier circuit.
- **Unresolved.** The historical parent averaged 11.43 phase-failing shots per 9,024; input-driven vs outcome-driven shares are unmeasured.
- **Fix.**
    - Keep R/Hmr rejected in the coherent profile.
    - In the optional measurement profile, a clean R consumes no entropy and a dirty R rejects before erasure.
    - Outcomes come from an independent evaluator secret indexed by trial, static Hmr ordinal and repetition; non-measurement padding must not advance a cursor.
    - Validate the corrected logical channel, with four outcome streams (one primary, three diagnostic).

## Yara Abu Saude: reply to C-03 (Oct 23)

Attachment: `Antigravity_Technical_Agent_Report.partial.md` (partial capture).

- **Agrees with:** the mechanism is real. Clean R and Hmr consume 8 bytes of the continued XOF, which can slide a later dirty reset's random phase kick under an artificially fixed seed.
- **Disputes:** that it is an open exploit. In the actual harness the inputs and XOF are derived from a Fiat-Shamir hash of the entire op stream, so padding scrambles both; and evading detection across 9,024 shots (141 batches of 64) is statistically impossible (p ≤ 2^−k).
- **On the fix:** clean resets consuming zero entropy plus dirty resets hard-rejecting closes the loophole. The indexed evaluator secret and multi-stream channel checker are "over-engineered".
