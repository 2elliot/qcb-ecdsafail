# Discord posts: reply to the modulus-specialization / Track G thread

Attach `reply_v0.2.md`.

## Full reply

```text
**Re: Modulus specialization / Track G**
Strong structure. Fresh-input error pricing is exactly what fixed test sets lack. Three changes would make it bite:

- **Price error over the attack, not per call.** ~28 additions run inside one coherent Shor run, and any failure means a rerun. Use Q·T / (1 − ε\_UB)²⁸. The leaderboard now formally allows "λ ≤ 18" expected failures per 9,024 shots (PR 419), i.e. ε ≈ 0.2%: about a 6% whole-attack penalty, not 0.2%. Top builds also strip gates that merely never fired in 121.6M samples (9,678 Toffolis in the incumbent), exactly the error your pricing should catch.
- **Size the corpus to what it must resolve.** At n = 10,000 a perfect circuit still gets ε\_UB ≈ 0.03%, 3× the references' ≈0.01% (2^-13.3); certifying that needs ~30k inputs. And ε\_max = 0.5% is 2.6× today's best. Tie it to attack success instead: 0.183% (95%) or 0.376% (90%).
- **Make the generator pure.** S3 limits its inputs but not its environment, and real builds do steer on named switches (our audit: 3616dbf2's build route depends on whether SUB4\_LEGACY\_POINT\_ADD is set). Fix: empty-env sandbox, plus a build-twice-under-different-env test.

Also: randomize the corner block within each class (fixed vectors are patchable). P-256's fold constant is 224 bits but weight 4, so the thresholds disagree. And say explicitly that secp256k1 specialization is fine for Track S.
Detailed doc (v0.2) attached.
```

## Follow-up (if the original v0.1 reply was already posted)

```text
**Follow-up from our harness audit:** two data points for this spec. (1) The leaderboard now formally allows "λ ≤ 18" expected failures per 9,024 shots (PR 419), i.e. ε ≈ 0.2%, about a 6% whole-attack penalty under K = 28. (2) Real builds do steer on named switches (3616dbf2's build route depends on whether SUB4\_LEGACY\_POINT\_ADD is set), so the pure-generator rule matters in practice.
```
