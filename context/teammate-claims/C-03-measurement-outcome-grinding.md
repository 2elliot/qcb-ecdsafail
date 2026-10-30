# Technical Specification: Measurement-outcome grinding

**Finding:** C-03 · **Revision:** 2.1 · **Audience:** implementation agent · **Priority:** P0

**Distribution:** private. **Status:** proposed v2 requirements; acceptance tests NOT_RUN. Publication requires a separate operator decision.

**Catalog severity:** High

**Evidence:** E2 two-qubit steering primitive with the pinned Simulator; E1 source mechanism; the input-driven versus outcome-driven share of observed phase failures is unmeasured.

This document contains a short human summary followed by an implementation-oriented Markdown specification. Code sketches define proposed behavior; they have not been integrated into or compiled against a v2 evaluator.

## 1. Executive summary — human-readable report

**Claim: Predictable measurement records remain selectable after inputs are fixed; use independent secret indexed outcomes and enforce separate reset and measurement-channel rules.**

**Problem**

The historical simulator continues the input XOF, and each R or Hmr consumes eight bytes. An inserted reset of an already-zero scratch qubit shifts later draws. A fixed public outcome sequence therefore still lets harmless padding change whether a later dirty reset produces a recorded phase failure.

**Demonstrated — two-qubit steering primitive**

- With one fixed RNG seed, inserting **0–15 clean resets** changed the final phase. Padding zero produced phase zero; padding two produced phase one. Every variant had **T=0** and clean final qubits.
- This establishes outcome steering in the primitive. It does not establish a complete invalid point adder or quantify the benefit for a frontier circuit.

**Unresolved**

- The historical parent averaged **11.43 phase-failing shots per 9,024**, overlapping its classical failures. The catalog did not separate failures caused by changed inputs from failures caused by changed outcomes.

**Why it matters:** Protecting input selection leaves a separate acceptance channel open if a contestant can predict or shift the outcome record.

**Fix**

- Keep R/Hmr rejected in the coherent profile. Keep the optional measurement profile disabled until its channel checker is validated; when enabled, a clean R consumes no outcome entropy and an active dirty R rejects before erasure.
- Give outcomes an independent evaluator secret. Index them by logical trial, static Hmr ordinal, and repetition; preserve these identities on exact retry. Nonmeasurement padding must not advance a sequential outcome cursor.
- Validate the corrected logical channel and allowed transcript, accepting correct measurement cleanup and input-independent branch-global signs. Use the specified four outcome streams as one primary plus three diagnostics; they supplement channel validation and do not multiply the independent-input count.

## 2. Scope and evidence

**C-03 — selectable measurement records. Priority P0.** Input secrecy alone does not protect an outcome record that is derived from contestant bytes or is predictable before commitment.

At the historical pin, the simulator continues the input XOF; R and Hmr each consume eight bytes. The catalog's E2 two-qubit primitive inserted 0–15 resets on an already-zero scratch wire before a dirty reset. Under a fixed seed, padding changed the final phase while all variants had zero Toffoli cost and clean final qubits. This proves a steering primitive, not a full invalid point adder. The historical 11.43 phase-garbage shots per packet overlap classical errors; their input-driven and outcome-driven shares remain unmeasured.

## 3. Proposed architecture and data flow

The trusted evaluator maintains an outcome master key independent of its input master key. Each immutable official job receives a separate derived seed for each outcome repetition. No outcome provider reads the input XOF or hashes contestant bytes to obtain its secret.

Within the frozen forward stream, Hmr instructions receive static ordinals starting at zero. The ordinal includes an instruction even when its classical condition is false. For a logical input index, ordinal, and repetition, the provider computes a keyed outcome bit. SIMD packing is an implementation detail: only active live lanes request outcomes; padding bits are zero.

Nonmeasurement identity padding cannot shift these ordinals. Added Hmr operations can shift subsequent ordinals, but the resulting record remains unknown before commitment. The design does not require equivalent circuits with different measurement schedules to receive identical records. Its security property is that contestants cannot choose a favorable known record; new official attempts remain budgeted.

The initial `coherent-unitary-v1` profile rejects R and Hmr. The optional `measurement-clean-v1` profile remains disabled until its channel checker is implemented and validated. In that profile, a safe clean R consumes no outcome draw, while an active dirty R rejects before erasure.

## 4. Formal invariants

For repetition r, the outcome seed is:

`HMAC-SHA256(outcome_master_key, LP("ecdsa-v2/outcome/v1") || LP(epoch_id) || U64BE(job_index) || U64BE(r))`.

For logical trial i and static Hmr ordinal e:

`outcome(i,e,r) = lowbit(first_byte(HMAC-SHA256(seed_r, LP("ecdsa-v2/outcome-bit/v1") || U64BE(i) || U64BE(e))))`.

LP and integer byte orders are those frozen in the central specification. Changing only outcome entropy must not change the input corpus. An exact infrastructure retry retains its original job and repetition identifiers.

For a reset target represented as a lane mask q, the enabled measurement profile requires:

`q & live_mask & active_condition == 0` before R.

This is a deliberately stronger contract than the historical stochastic reset. It does not apply to Hmr targets.

For each measurement outcome record m and fixed classical offset O, the allowed corrected branch satisfies:

`K_m = c_m(O) U_O`, with `sum_m |c_m(O)|² = 1`.

The coefficient is independent of the unknown target state. Input-independent branch-global signs are allowed; a blanket requirement that sampled phase bits equal zero is wrong for this profile. Relative phase or transcript dependence on logical data remains a defect.

Full-circuit sampling uses four total trajectories per input when this profile is enabled: one predesignated primary and three diagnostics. The statistical denominator remains 100,000 independent primary input draws. Any detected defect rejects. Repetition is a detection aid, not a channel-equivalence proof.

## 5. Reference algorithm

The following **illustrative pseudocode is not compiled**. It defines proposed adapter responsibilities, not existing simulator methods.

```text
PrepareSchedule(frozen_ops, profile):
    if profile == coherent_unitary_v1:
        RejectAny(R, Hmr)
    next_hmr := 0
    for op in frozen_ops:
        if op.kind == Hmr:
            op.static_ordinal := next_hmr
            next_hmr := next_hmr + 1
    return frozen_schedule

OutcomeBit(job, repetition, logical_trial, hmr_ordinal):
    seed := DeriveOutcomeSeedFromTrustedKey(job, repetition)
    msg := LP("ecdsa-v2/outcome-bit/v1") ||
           U64BE(logical_trial) || U64BE(hmr_ordinal)
    return HMAC_SHA256(seed, msg)[0] & 1

ExecuteReset(target, active_mask, live_mask, profile):
    require profile == enabled_measurement_clean_v1
    relevant := active_mask & live_mask
    if (target & relevant) != 0:
        fail DIRTY_RESET
    target := target & ~relevant
    consume_zero_outcome_bits()

CheckBoundedMeasurementGadget(gadget, role_manifest):
    branches := ExactIndependentKrausOracle(gadget, role_manifest)
    require ScratchEndsZero(branches)
    require OnlyDeclaredClassicalRecords(branches)
    for branch in branches:
        require ExistsDataIndependentScalar(
            branch.operator, expected_logical_operation)
    require SumBranchProbabilityOperators(branches) == Identity
```

The last routine is meaningful for the supplied small gadget; a scalable full-circuit implementation is not supplied. Until certified composition or another explicitly versioned assurance method exists, the measurement profile stays disabled.

The positive gadget is `CX q0 q1; HMR q1 b0; Z q0 if b0`. Its corrected branch operator on q0 is `I/sqrt(2)`. Omitting Z gives `I/sqrt(2)` and `Z/sqrt(2)`, which fails the identity-channel relation on coherent inputs. Appending unconditional Neg or outcome-conditioned Neg changes only branch-global signs and must remain accepted.

## 6. Reproducible fixtures and expected outcomes

**All v2 tests are NOT_RUN.** Existing support, relative to the bundle root, includes `fixtures/entropy-kat.json`, `fixtures/micro-circuit-cases.json`, `fixtures/reset-clean.kmx`, `fixtures/reset-dirty.kmx`, `fixtures/hmr-correct-uncompute.kmx`, and `fixtures/hmr-missing-fixup.kmx`. Additional profile controls are `fixtures/hmr-global-neg.kmx` and `fixtures/hmr-outcome-neg.kmx`.

| Test / fixture | Controlled procedure | Required outcomes |
|---|---|---|
| **AT-C03-01 / FX-C03-01** | Apply dirty-reset and clean-reset variants under zero, one, and alternating masks; prepend 0–15 clean resets on another scratch wire. | Legacy erasure can hide the defect on a zero mask. Enabled measurement v2 rejects every active dirty R independently of padding; safe clean R passes without consuming entropy. Coherent admission rejects R. |
| **AT-C03-02 / FX-C03-02** | Hold input entropy fixed while changing outcome entropy; then hold outcome authority fixed while changing input entropy. Inspect provider reads and KAT results. | Input vectors cannot depend on the outcome provider. Production rejects public fixture overrides. A valid channel succeeds under every defined outcome record. |
| **AT-C03-03 / FX-C03-03** | Evaluate corrected, missing-correction, and branch-global-sign gadgets with a bounded coherent oracle. | Corrected and global-sign controls pass the enabled channel profile; missing correction fails deterministically. The coherent profile rejects Hmr structurally. |

The role manifest declares q0 logical data, q1 initially/finally zero scratch, and b0 a permitted measurement transcript. Test basis states and coherent states `|+>` and `|+i>`, or an equivalent exact Kraus/reference-system oracle. Merely checking the two basis outputs is insufficient.

The explicit masks do not reproduce the catalog's original seeded padding log without its historical seed artifact. They provide deterministic regression inputs for the same mechanism. No adapter or general full-circuit channel checker has been run.

## 7. Operational impact

Maintainers need an independently keyed, indexed outcome provider and profile-aware dispatch. Logical indexing avoids batch-layout dependence and permits exact internal replay. Contestants retain valid measurement cleanup only under the enabled validated profile; legacy reset usage requires compatibility review. Four total trajectories increase execution work, but no measured runtime multiplier is claimed.

## 8. Implementation checklist

- [ ] Implement the specified seed domains, integer encoding, and static Hmr numbering.
- [ ] Separate logical input indices from SIMD lanes and exclude padded lanes.
- [ ] Reject dirty R before erasure; preserve safe R only in the enabled measurement profile.
- [ ] Validate the role manifest and permitted transcript outputs.
- [ ] Accept correct branch-global signs and reject input-dependent phase/transcript defects.
- [ ] Keep the measurement profile disabled until its checker and controls are implemented.
- [ ] Run and retain all three regression outcomes before changing NOT_RUN.

## 9. Evidence limits and references

The catalog establishes steerability in a minimal primitive. It does not attribute a quantified portion of frontier error to outcome steering. For independent streams with per-input detection probability d, survival is `(1−d)^m`; `2^-m` applies to the fair-coin primitive and is an upper bound only if d is at least one half.

- **Catalog:** C-03, C-08, §8 P2/P9; lanes CH-grind-4, CH-grind-102, CH-exact-1.
- **Artifact:** `.run/grind/cheese/implementer/rng.log`, sha256 `b5fe5001…`.
- **Pinned sources:** [evaluator](https://github.com/Layr-Labs/ecdsafail-challenge/blob/917c33d1f0d2d1430d48cfbd4ca836f72889b2f7/src/bin/eval_circuit.rs) and [simulator](https://github.com/Layr-Labs/ecdsafail-challenge/blob/917c33d1f0d2d1430d48cfbd4ca836f72889b2f7/src/sim.rs). These inspected files match historical pin `0ae6d109bee6f3241e1f74c38fddb5a05b7f79de`; no new PoC claim follows.

### Shared contract and supporting files

Apply [acceptance-spec.md](acceptance-spec.md) as the shared mathematical, ABI, profile, entropy, statistical, and ranking contract. Relative `fixtures/` paths resolve from the individual-report bundle root. The bundle supplies the existing fixture data and generator; fixture names in a Markdown report do not embed those files. The v2 adapter and a pinned compatible full-domain native reversible positive-control circuit remain outstanding prerequisites.

The default `coherent-unitary-v1` profile rejects R/Hmr. `measurement-clean-v1` remains disabled until its channel checker is validated. Its four outcome streams comprise one predesignated primary and three diagnostics; they do not multiply the number of independent input draws. Known reproducible defects reject regardless of a clean sampled ledger.
