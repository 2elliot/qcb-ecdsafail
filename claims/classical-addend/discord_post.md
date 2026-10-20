# Discord post: classical-addend claim

Thread title: **Classical addend: restrict offset bits to loads**

Paste-ready text (Discord markdown; underscores are escaped). Attach `claim_v0.1.md` and `audit/audit_report.md`.

```text
**Claim: The classical-addend surface — unused by the earlier leaders, now mined by the current ones. Restrict offset bits to loads.**

**Problem**
The harness hands the addend A to the circuit as classical bits: gates conditioned on them only count when they fire, and classical bit logic is free. Windowed Shor (28 additions per attack) gets A as *quantum* data from a table lookup, so none of that transfers.

**Measured** (our audit, harness 3dee0cd)
- Earlier leaders don't use it: 0 addend-conditioned Toffolis in 8e9c9a2, 3616dbf2 and 3493c837 (lowest-Q promoted). A only enters 5 modular add/subs (~1,600 T); total specialization ≤ ~0.75k T.
- The paper's +29.6% windowed overhead is lookup cost (≈ 5.87 table traversals), not a hidden discount.

**Emerging** (source-read, not built)
- The current top-3 lineage conditions arithmetic directly on offset bits: first carries by construction, plus "classical-factor" passes in ranks 1–2 (claimed 7.5 T). Too small to flip the top 3.
- It also feeds x\_A into the division seam with no qubit register. A windowed port must hold A in qubits there: up to +256 qubits if that's near the peak (unmeasured).

**Why it matters:** the leaderboard shows a 37% drop in S since 8e9c9a2. Under windowed accounting it's 31% at best, and as little as 6% if the leader needs 5 lookups plus that width.

**Fix**
- Offset bits may only drive X-loads into qubits (plus classical bit ops for derived constants like 3·x\_A, which become table columns). Anything else conditioned on an offset bit is rejected: a static check that runs in seconds with our taint tool.
- Report S\_W = (Q + 16) × (T + k·(2¹⁶ − 2) + 512k) alongside S, with k = number of addend-access windows.

Claim doc + full audit report attached.
```
