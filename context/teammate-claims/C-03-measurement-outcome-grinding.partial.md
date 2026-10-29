<!-- PARTIAL capture, 2026-10-29, from the Discord attachment viewer (Jack, thread "Predictable measurement records remain selectable after inputs are fixed.", Oct 22). Sections 1–2 are complete, sections 3–4 are partial, and only the opening lines of 5–9 were captured. Gaps are marked [not captured]. Download the original (14 KB) to replace this file. -->

# Technical Specification: Measurement-outcome grinding

**Finding:** C-03 · **Revision:** 2.1 · **Audience:** implementation agent · **Priority:** P0
**Distribution:** private. **Status:** proposed v2 requirements; acceptance tests NOT_RUN. Publication requires a separate operator decision.

## 1. Executive summary — human-readable report

**Claim:** Predictable measurement records remain selectable after inputs are fixed; use independent secret indexed outcomes and enforce separate reset and measurement-channel rules.

**Problem.** The historical simulator continues the input XOF, and each R or Hmr consumes eight bytes. An inserted reset of an already-zero scratch qubit shifts later draws. A fixed public outcome sequence therefore still lets harmless padding change whether a later dirty reset produces a recorded phase failure.

**Demonstrated — two-qubit steering primitive.**
- With one fixed RNG seed, inserting 0–15 clean resets changed the final phase. Padding zero produced phase zero; padding two produced phase one. Every variant had T = 0 and clean final qubits.
- This establishes outcome steering in the primitive. It does not establish a complete invalid point adder or quantify the benefit for a frontier circuit.

**Unresolved.** The historical parent averaged 11.43 phase-failing shots per 9,024, overlapping its classical failures. The catalog did not separate failures caused by changed inputs from failures caused by changed outcomes.

**Why it matters:** protecting input selection leaves a separate acceptance channel open if a contestant can predict or shift the outcome record.

**Fix.**
- Keep R/Hmr rejected in the coherent profile. Keep the optional measurement profile disabled until its channel checker is validated; when enabled, a clean R consumes no outcome entropy and an active dirty R rejects before erasure.
- Give outcomes an independent evaluator secret. Index them by logical trial, static Hmr ordinal, and repetition; preserve these identities on exact retry. Nonmeasurement padding must not advance a sequential outcome cursor.
- Validate the corrected logical channel and allowed transcript, accepting correct measurement cleanup and input-independent branch-global signs. Use the specified four outcome streams as one primary plus three diagnostics; they supplement channel validation and do not multiply the independent-input count.

## 2. Scope and evidence

- C-03 covers selectable measurement records (priority P0). Input secrecy alone does not protect an outcome record that is derived from contestant bytes or is predictable before commitment.
- At the historical pin, the simulator continues the input XOF; R and Hmr each consume eight bytes. The catalog's E2 two-qubit primitive inserted 0–15 resets on an already-zero scratch wire before a dirty reset.
- Under a fixed seed, padding changed the final phase while all variants had zero Toffoli cost and clean final qubits. This proves a steering primitive, not a full invalid point adder.
- The historical 11.43 phase-garbage shots per packet overlap classical errors; their input-driven and outcome-driven shares remain unmeasured.

## 3. Proposed architecture and data flow

- **Separate keys.** The trusted evaluator maintains an outcome master key independent of its input master key. Each immutable official job receives a separate derived seed for each outcome repetition.
- **No outcome provider hashes contestant bytes to obtain its secret.** [a short passage is not captured]
- **Static ordinals.** Within the frozen forward stream, Hmr instructions receive static ordinals starting at zero. The ordinal includes an instruction even when its classical condition is false. [a short passage is not captured]
- **Keyed outcome bits.** For a logical input index, ordinal, and repetition, the provider computes a keyed outcome bit. SIMD packing is an implementation detail: only active live lanes request outcomes; padding bits are zero.
- **What padding can and can't do.** Nonmeasurement identity padding cannot shift these ordinals. Added Hmr operations can shift subsequent ordinals, but the resulting record remains unknown before commitment.
- **Security property.** The design does not require equivalent circuits with different measurement schedules to receive identical records. Its security property is that contestants cannot choose a favorable known record; new official attempts remain budgeted.
- **Profiles.** The initial coherent-unitary-v1 profile rejects R and Hmr. The optional measurement-clean-v1 profile remains disabled until its channel checker is implemented and validated. In that profile, a safe clean R consumes no outcome draw, while an active dirty R rejects before erasure.

## 4. Formal invariants

For repetition r: [opening not captured]

- **Outcome bit** (function name partially captured):
  `outcome(i, e, r) = lowbit(first_byte(HMAC-SHA256(seed_r, LP("ecdsa-v2/outcome-bit/v1") || U64BE(i) || U64BE(e))))`
- **Encoding.** LP and integer byte orders are those frozen in the central specification.
- **Isolation.** Changing only outcome entropy must not change the input corpus. An exact infrastructure retry retains its original job and repetition identifiers.
- **Reset contract.** For a reset target represented as a lane mask q, the enabled measurement profile requires `q & live_mask & active_condition == 0` before R. This is deliberately stronger than the historical stochastic reset, and does not apply to Hmr targets.
- **Measurement branches.** For each measurement outcome record m and fixed classical offset O, the allowed corrected branch satisfies `K_m = c_m(O) U_O`, with `sum_m |c_m(O)|² = 1`. The coefficient is independent of the unknown target state. Input-independent branch-global signs are [not captured]

## 5. Reference algorithm

"The following illustrative pseudocode is not compiled. It defines proposed adapter responsibilities, not existing simulator methods." The pseudocode begins with `PrepareSchedule(frozen_ops, profile)`, branching on `profile == coherent_unitary_v1`. [rest not captured]

## 6. Reproducible fixtures and expected outcomes

"All v2 tests are NOT_RUN." Existing support, relative to the bundle root, includes `fixtures/entropy-kat.json`, `fixtures/micro-circuit-cases.json`, `fixtures/reset-clean.kmx` and `fixtures/reset-dirty.km…` [rest not captured]

## 7. Operational impact

"Maintainers need an independently keyed, indexed outcome provider and profile-aware dispatch. Logical indexing avoids batch-layout dependence and permits exact internal replay. Contestants retain valid measurement clea[nup] …" [rest not captured]

## 8. Implementation checklist (first items captured)

- [ ] Implement the specified seed domains, integer encoding, and static Hmr numbering.
- [ ] Separate logical input indices from SIMD lanes and exclude padded lanes.
- [ ] Reject dirty R before erasure; preserve safe … [rest not captured]

## 9. Evidence limits and references

- **Evidence limit.** The catalog establishes steerability in a minimal primitive. It does not attribute a quantified portion of frontier error to outcome steering.
- **Survival probability.** For independent streams with per-input detection probability d, survival is (1−d)^m. 2^-m applies to the fair-coin primitive, and is an upper bound only if d is at least one half.
- **References.**
    - Catalog: C-03, C-08, §8 P2/P9; lanes CH-grind-4, CH-grind-102, CH-exact-1.
    - Artifact: .run/grind/cheese/implementer/rng.log, sha256 b5fe5001….
    - Pinned sources: evaluator and simulator. These inspected files match historical pin 0ae6d109bee6f3241e1f74c38fddb5a05b7f79de; no new PoC claim follows.
- **Shared contract.** Apply acceptance-spec.md as the shared mathematical, ABI, profile, entropy, statistical, and ranking contract. Relative fixtures/ paths resolve from the individual-report bundle root. [rest not captured]
