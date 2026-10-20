# QCB audit brief: classical-addend interface gap (ecdsa.fail)

This file is context for Claude Code. It sets up an AUDIT of this repository for a research claim by QCB (UC Berkeley quantum computing club). Everything below about how the code behaves is a hypothesis until you verify it in the code.

## 0. Ground rules (read first)

1. **Audit only.** Do NOT edit, delete, or reformat any tracked file. Do NOT commit, push, open PRs, or run `ecdsafail login` / `ecdsafail submit`.
2. **Write all outputs under `qcb_audit/`** (untracked). The main deliverable is `qcb_audit/audit_report.md`.
3. **Evidence standard.**
   - Every finding cites `path:line`, plus the commit hash if it is not on HEAD.
   - Anything you did not read in code or measure in a run is labeled `INFERRED`.
   - Numbers you did not measure carry their source (e.g. "paper §5.2.3").
4. **No new dependencies.** No network access beyond `git fetch` of this repository. If a step needs anything else, stop and ask.
5. **Ask before heavy runs.** Ask before any command expected to run more than 10 minutes or use more than 8 GB RAM. The README warns that code here comes from many contributors, so prefer reading over running, and build only when a task needs a measured number.
6. **Work in phases.** After each task (V1, V2, ...), append findings to the report and STOP for human review. Do not start the next task until told.
7. **The code wins.** If this brief conflicts with what the code does, report the conflict; do not paper over it.

## 1. What we are trying to establish

### Hypothesis (unverified)

The harness gives the addend A = (x_A, y_A) to the circuit as CLASSICAL bits (`offset_x`, `offset_y`). A circuit can therefore:

- (a) classically condition gates on A, so a Toffoli with an addend-bit control becomes a conditionally executed CNOT (no Toffoli);
- (b) precompute addend-derived constants such as 3·x_A mod p for free;
- (c) write A into qubits with classically conditioned X gates.

The reported T is the average EXECUTED Toffoli count, so conditioned gates that do not fire are not counted.

### Why that matters

Full-attack estimates use windowed Shor: w = 16, 28 point additions per attack. There, A is selected by a quantum address through a table lookup (QROM). A is quantum data, so (a)–(c) are unavailable. The leaderboard may therefore reward optimizations that do not transfer to a real attack.

### Known numbers

These come from arXiv:2609.09582 (the ECDSA.Fail paper) and are NOT yet verified here.

| Item | Value |
|---|---|
| Incumbent 8e9c9a2 (promoted source commit 60d61859fa69) | Q = 1,151, T = 1,299,453 |
| Its windowed-compatible variant (paper §5.2.3, Supplemental Note F; the paper says the code is in this repo) | Q = 1,162, T = 1,684,161 |
| Measured overhead of the variant | +11 qubits, +384,708 Toffolis (+29.6%) |
| Standard analytic accounting used in the literature | +16 qubits, +3·2^16 = 196,608 Toffolis per call |
| Excess of measured over analytic | 188,100 Toffolis (note 384,708 / 2^16 ≈ 5.87) |

How the paper describes the variant:

- It loads A through QROM in 3 stages.
- It clears the lookup payload by "exact reversible replay".
- It removes routing ancillas by measurement-based uncomputation.
- It handles the A = O (point at infinity) table row by conditioning arithmetic on a nonzero-address predicate.

### The key open question

Which of these explains the +188,100 excess?

- **R1, hidden discount:** lost constant propagation or classical conditioning that the leaderboard never pays for.
- **R2, lookup overhead:** for example ~6 full QROM traversals (if each unload is a full reverse traversal), plus the nonzero-address conditioning.

The claim's headline depends on the answer. Report evidence for whichever reading holds, or a numeric split if it is a mix.

## 2. Tasks

### V1: Mechanism (read-only, no builds)

The README lists the contract files as `src/main.rs`, `src/circuit.rs`, `src/sim.rs`, `src/weierstrass_elliptic_curve.rs`. Also find the evaluator, possibly `src/bin/eval_circuit.rs`.

a. How are `target_x`, `target_y`, `offset_x`, `offset_y` represented (qubits vs classical bits)? Where are the offsets loaded per shot?
b. How are classically conditioned operations encoded (field names, op types)? How does the simulator execute them?
c. How is the reported Toffoli count computed? Is a CCX/CCZ whose classical condition is false excluded? Is it averaged over shots?
d. Is there any operation that computes on classical bits without being counted as a Toffoli? Can measurement outcomes be written to classical bits and used as conditions?
e. How are the 9,024 test inputs and the measurement randomness seeded?

Deliverable: a short table answering a–e with `path:line`.

### V2: Exposure in the incumbent

**Find the incumbent source.**
- Try `git cat-file -t 60d61859fa69`.
- If that fails, try `git fetch --all`, `git log --all --oneline | grep -i 8e9c9a2`, `results.tsv`, and notes under `src/point_add/memory/`.
- If you still cannot find it, use the current HEAD of `src/point_add/` and say so in the report.

**Then answer:**

a. Count the CCX/CCZ ops whose execution depends on an offset bit, either directly as a classical condition or via a classical bit derived from the offsets.
b. Estimate the executed Toffolis these account for, and what a quantum-controlled equivalent (A held in qubits) would cost. A rough per-site estimate is fine if labeled `INFERRED`.
c. List addend-derived values that are computed classically or at build time (e.g. 3·x_A mod p), and how they enter the circuit.
d. If findable, repeat a–c briefly for the post-cutoff submission 3616dbf (ping-pong GCD).

If an instrumented count needs code, write it as a NEW file under `qcb_audit/`, such as a small Rust bin or Python script that reads the op stream. Never modify the harness.

### V3: Decomposition of the windowed overhead (the decisive task)

Find the windowed-addition-compatible variant (paper Supplemental Note F). Search branches, directories, and notes for "window", "qrom", "lookup", "unlookup".

Split its +384,708 average Toffolis versus the incumbent into:

1. the number of QROM traversals and the Toffolis per traversal;
2. the unload method (reverse traversal vs measurement-based) and its Toffoli cost;
3. the cost of the nonzero-address (A = O) conditioning;
4. the residual arithmetic delta from losing classical specialization.

The four parts must reconcile to the measured total within a stated tolerance. State which reading (R1, R2, or a mix) the evidence supports.

### V4: Ranking sensitivity (only after V3)

For the top 3 promoted circuits you can find, estimate a windowed score. Say whether the ranking could change under a windowed-interface score. Rough estimates are fine if labeled.

### V5: Google comparison (low priority, read-only)

Does anything in the repo (credits, Zenodo-derived code, notes) show whether Google's ZK-proven circuits (arXiv:2603.28846) used the same classical-addend interface? Report only what the evidence shows.

## 3. Critique pass (after V3)

List the 3 strongest objections to the claim given your findings. Mark each as resolved, partially resolved, or fatal. Objections to check:

- **Double-and-add uses a classical addend legitimately.** Counter-argument: it needs 512 controlled calls versus 28 windowed calls. Check this reasoning against the actual interface.
- **The overhead is mostly lookup cost**, already priced by the analytic formula (reading R2).
- **Random offset masks remove the A = O cost.** arXiv:2609.05625 adds random offsets to the tables, so the A = O handling cost may disappear.
- **The V1 mechanism differs from this brief.**

## 4. Report format (`qcb_audit/audit_report.md`)

- **Header:** commit audited, date, tool and model used.
- **One section per task:** findings (with `path:line`), measured numbers, `INFERRED` items, open issues.
- **Final section:** verdict on R1 vs R2 with numbers, and any kill/pivot flags from §5.

## 5. Kill / pivot criteria (flag prominently if any hold)

- **K1:** the offset bits are not classical bits, or cannot be used as classical conditions. The mechanism fails and the claim must be reframed.
- **K2:** the executed-Toffoli counter counts conditioned Toffolis regardless of their condition. The "skipped Toffoli" part fails, though build-time constant propagation may still apply.
- **K3:** V3 shows part 4 (residual specialization delta) is about zero. The specialization discount is negligible for the incumbent, and the claim pivots to "interface mismatch risk" instead of a measured discount.

## 6. Context files in `qcb_audit/context/`

- `robust_architecture_specification.md`: teammate claim 1 (test-set integrity / Fiat-Shamir).
- `ecdsafail_modulus_specialization_AGENT_SPEC_v0_2.md`: teammate claim 2 (Track G, modulus families).

Identifiers in these specs are partly assumed. Verify them before relying on them.

## 7. Glossary

- **Toffoli:** a CCX gate; CCZ also counts as one.
- **Q:** peak live qubits.
- **T:** average executed Toffolis per shot.
- **S:** Q × T.
- **Addend / offset:** the point A added to the quantum accumulator R.
- **QROM lookup:** loads table entry T[j] into a register using a quantum address |j⟩.
- **Unlookup:** clears that register afterwards.
- **Windowed Shor:** 28 additions per attack, each with A loaded by QROM from a 2^16-entry table.
