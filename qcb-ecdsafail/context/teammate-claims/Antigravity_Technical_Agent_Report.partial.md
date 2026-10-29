<!-- PARTIAL capture, 2026-10-29, from the Discord attachment viewer of "Antigravity Technical Agent Report.txt" (Yara Abu Saude's reply to C-03, Oct 23). Math rendered by Discord was flattened. Gaps are marked [not captured]. Download the original (7 KB) to replace this file. -->

# Technical Report for Agents (reply to C-03)

## 1. Codebase Verification & Citations

**XOF continuation and entropy consumption: verified.**
- `src/bin/eval_circuit.rs:204-218`: `fiat_shamir_seed(ops)` initializes a single Shake256Reader.
- `src/bin/eval_circuit.rs:248-254`: `run_tests` consumes 9024×64 bytes from this XOF to sample the curve points (targets and offsets).
- `src/bin/eval_circuit.rs:273`: the same stream is passed by reference: `Simulator::new(total_qubits as usize, num_bits as usize, &mut xof)`.
- `src/sim.rs:140-155`: both `OperationType::Hmr` (lines 140–148) and `OperationType::R` (lines 149–155) unconditionally call `self.xof.read(&mut buf)` with `[0u8; 8]`, consuming 8 bytes per executed operation per 64-shot batch.

**Clean reset behavior: verified.** `src/sim.rs:150-154`: for a clean qubit (`self.qubit(op.q_target) == 0`), phase ← phase ⊕ (0 ∧ rng_val ∧ cond) = phase. The qubit remains 0, no phase kickback occurs, and self.stats.toffol… [not captured]

**Dirty reset phase scrambling: verified.** [details not captured]

## 2. Empirical Verification of the Steering Primitive

- **Setup.** A reproduction unit test against `quantum_ecc::sim::Simulator` with 2 qubits (q0 = |0⟩, q1 = |1⟩): P ∈ [0, 15] clean resets `R q0`, followed by 1 dirty reset `R q1`.
- **Case A (external fixed seed).** Seeded with a fixed constant XOF: P = 0 produced phase = 1, P = 3 produced phase = 0, P = 5 … [not captured]
- **Further cases.** [not captured]

## 3. Strongest Critiques & Gaps in the Claim

**The Fiat-Shamir binding gap ("fatal vulnerability" in the claim).**
- **The claim says:** "Predictable measurement records remain selectable after inputs are fixed... A fixed public outcome sequence therefore still lets harmless padding change whether a later dirty reset produces a recorded phase failure."
- **Critique:** in this benchmark harness, inputs and measurement records are not fixed independently of the circuit ops.
    - In `src/bin/eval_circuit.rs:204-218`, the seed is Seed = SHAKE256("quantum_ecc-fiat-shamir-v2" ∥ |ops| ∥ ops[0] ∥ … ∥ ops[n−1]).
    - Inserting even a single R operation alters ops.len() and appends 56 bytes to the preimage. Due to SHAKE256's avalanche effect, modifying padding does not "slide a fixed public sequence"; it completely reshuffles the 9,024 test points (T, O) generated at lines 248–269.
    - A contestant cannot target a kn[own] … [rest not captured]
- **Detection.** (From the Discord post) Evading detection across 9,024 shots (141 batches of 64) requires all dirty-shot bits to collide with zeros in the random draws: p ≤ 2^−k.

## 4. Assessment of the Proposed Fix

- **Clean R consuming 0 entropy: effective and minimal.** If `src/sim.rs:149-155` is updated so R does not draw from self.xof when qubit == 0, stream sliding is eliminated.
- **Dirty R hard-rejecting before erasure: optimal fix.** Rather than relying on randomized phase kickback (`self.phase ^= qubit & rng_val`), the simulator should immediately panic or return Err if (self.qubit(op.q_target) & cond … [not captured]
- **Other parts of the fix.** (From the Discord post) The indexed evaluator secret and multi-stream channel checker are "over-engineered for what a simple deterministic dirty check already solves."

## 5. Unverified Items Flagged

- **"11.43 phase-failing shots per 9,024":** could not be verified in this repository. Git history shows prior c… [rest not captured]
