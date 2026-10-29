# Draft reply: P003 (per-run error pricing)

**Status:** POSTED 2026-10-20, 9:22 PM (the earlier text: k = 28, offset-keyed patching ~0 T, shared secret corpus, nits). The zero-cost T̄ point below was never posted. Aodhfionn has since replied to P003 and to this post (amplitude exponent; see `notes/verified-claims-catalogue-2026-10-29.md`). **Their doc:** `context/teammate-claims/P003_per_run_error_pricing__AGENT.md`.

## What they say

Score = T̄·Q·(1 − ε_UB)^−k, with k = 32 inferred from Litinski's windowing (range 13–32).

- **Measured on leader 7499c90** (933,231,010; T 756,265; Q 1,234), 100,032 fixed-seed inputs (`pfs fixed`, `public-seed-A`): ε̂ = 0.208% any-channel, ε_UB 0.233%.
- **At k = 32:** ×1.078, scored 1,005,624,757.
- **Sampling noise:** N ≥ 2·ln(G)·k²·ε/δ². Proposes adaptive N and ε_max ≈ 0.33%.

## What we re-checked (Python, 2026-10-22)

Every number reproduces:
- the CP bounds (0.2332% for 208/100,032; references 0.0029912 and 0.0062823);
- the scored values 935,404,015, 961,984,966 and 1,005,624,757;
- the run-failure rates and ε_max ≈ 0.33%;
- the noise and N estimates (3.9 × 10⁹ shots ≈ 7 days; 1.9 × 10⁷ ≈ 50 min).

One nit: the formula says round(T̄·Q·…), but the table rounds T̄ first (933,222,372 vs 933,222,875).

## Points to make

1. **k = 28, not 32.** Sourced from Google's whitepaper (App. A.3) and the trapped-ion estimate. At k = 28 the leader's penalty is ×1.068 (996.3 M), and the 90% gate becomes ε_max ≈ 0.376%.
2. **A fixed public corpus defeats the pricing, and the cost measurement too.** Their prototype uses `public-seed-A`. Under the classical-addend ABI, recognizing a public input is free (positions list, item 3), so:
    - every failing error-corpus point is patched for 0 T, giving f → 0;
    - the arithmetic can be switched off on cost-corpus shots, giving T̄ ≈ 0.
3. **One shared secret corpus per epoch.** This keeps their own §4 observation (identical failure behaviour gives identical f) while removing grinding.
4. **The f = 0 floor is harmless.** ε_UB's ×1.0001 floor at k = 32 is identical for every exact circuit, so it can't change a ranking.

## Discord draft

```
**Re: P003 — Per-run error pricing**
+1, this makes the composition point concrete, and the leader measurement is great. I re-checked every number (CP bounds, multipliers, scored values, noise and N estimates) and they all reproduce. Suggestions:

- **Use k = 28, not 32.** Google's whitepaper (App. A.3) gives 28 windowed point additions at w = 16, and the trapped-ion estimate union-bounds over 28: sourced rather than inferred. Effect on the leader: ×1.068 instead of ×1.078 (996.3M), and the 90%-success gate becomes ε\_max ≈ 0.376%.
- **A fixed public corpus undoes the pricing, and the cost measurement too.** With `public-seed-A` fixed and classical addends, a circuit can recognize any corpus input for free (stacked classical conditions; condition-false Toffolis aren't counted). It can patch every failing error-corpus shot for 0 Toffolis (f → 0), and switch its arithmetic off on cost-corpus shots while XORing in precomputed answers (T̄ → 0). Inferred from our audit's V1 semantics; not built yet. The corpus has to be secret and fresh, or the addend has to be quantum.
- **One shared secret corpus per epoch.** Your last §4 bullet is the key: on a shared corpus, circuits with the same failure behaviour get identical f. Score every submission in an epoch on one secret corpus: no grinding, and near-identical circuits compare almost exactly.

Nits: the formula says round(T̄·Q·…) but the table rounds T̄ first (933,222,372 vs 933,222,875). And the f = 0 floor (×1.0001) is the same for every exact circuit, so it's fine to keep.
```
