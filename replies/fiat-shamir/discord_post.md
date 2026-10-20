# Discord posts: reply to "Eliminating Fiat-Shamir tail-nonce grinding"

Attach `reply_v0.2.md`.

## Full reply

```text
**Re: Eliminating Fiat-Shamir tail-nonce grinding**
Agree with the diagnosis, and our harness audit confirms it: top submissions pin a 48-bit tail nonce in code, and the incumbent's own notes expect ~10 output mismatches per 9,024-shot run. But a fixed public test set has its own exploit.

- **Fixed public points are patchable.** You don't need to memorize all 10,512 points, only the ~20–100 your approximate circuit gets wrong. A 24-bit pattern check plus a CNOT correction costs ~46 Toffolis per point, so even a 1%-error circuit passes for <0.4% of T. The economic proof only rules out full lookup tables.
- **The 15% figure doesn't add up.** Pass probability is (1 − ε)⁹⁰²⁴; a 48-bit nonce hides at most ~0.37% (15% gives 2⁻²¹¹⁶). The leaderboard's own "λ ≤ 18" approximation contract (PR 419) is ε ≈ 0.2%.
- **The canonicalizer isn't needed** once test points don't depend on the op stream (and X(a) X(b) X(a) X(b) slips past it anyway).
- **Two of the three exact checks already exist.** Ancilla and phase checks run every shot; the README's forward∘reverse check isn't implemented, so the adjoint pass is new, but it only adds information for measurement/reset ops.
- **Measurement randomness comes from the same hash stream as the test inputs** in the current harness. Under a static public seed it would be fixed and public too, so it needs a secret seed.
- **Exceptional cases:** the task domain already excludes R ∈ {A, −A, −2A} (−2A is missing from Tier 1). Real Shor estimates use random offset masks, not complete adders.

**Proposed merge:** keep the diagnosis and per-shot checks. Draw *all* inputs (corner cases too, randomized within each class) fresh from a post-window seed, as in Track G. Use secret measurement randomness, plus a public dev seed for local iteration.
Detailed doc (v0.2) attached.
```

## Follow-up (if the original v0.1 reply was already posted)

```text
**Follow-up from our harness audit (doc v0.2 attached):** correction: the README's forward∘reverse check isn't implemented at HEAD (the scorer runs one forward pass), so the adjoint pass *is* new work, though it only adds information for measurement/reset ops. Two confirmations: measurement randomness comes from the same hash stream as the test inputs, and top submissions pin a ground tail nonce in code.
```
