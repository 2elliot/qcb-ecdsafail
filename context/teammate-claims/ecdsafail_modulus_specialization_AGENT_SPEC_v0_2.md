# AGENT SPEC: Track G family evaluation (ECDSA.Fail) v0.2 (compact)

Status: DRAFT. Repo not inspected. `[A]` = assumed identifier: verify, record mapping in `docs/trackg_mapping.md`. MUST/SHOULD per RFC 2119. Rationale: `ecdsafail_modulus_specialization_SPEC.md` §0.

## 0 Rules
1. Contract = §3-§6; acceptance = §7. Start with T0.
2. Do not invent names/APIs. Unknown `[A]` -> find real equivalent, record mapping.
3. Prime/curve constants: take from NIST FIPS 186-5 / SEC 2, verify independently (primality, claimed form), report the check. Never trust constants in this doc.
4. Track S (existing) behavior, strings, exit codes: unchanged.
5. Spec/code conflict -> stop and report. Do not pick silently.
6. "Done" = §7 tests run this session, output quoted.

## 1 Problem
Score S=Q*T on fixed p=2^256-2^32-977; approximation allowed. Winning generators embed p's shape: reduction/halving/doubling cells and Karatsuba fold use 2^256≡2^32+977; iteration budgets and width bounds are tuned to the validation support. Unpriced, so free score, and it says little about modular inversion in general.
NOTE: the GCD core (jump-2, ping-pong) needs only odd p. Specialization is in reduction cells, budgets, calibrated bounds. Code comments/docs MUST preserve this distinction.
Goal: Track G ranks a generator over a modulus family incl. hidden members, with error priced. Track S unchanged.

## 2 Pipeline
`build(ModulusParams)` -> ops -> canonicalize -> per instance: cost corpus -> Q,T; fresh inputs -> fails (any channel) -> eps_UB -> `S_adj=Q*T/(1-eps_UB)`; gate `eps_UB<=eps_max`; `GScore=geomean(S_adj)` over ranked.

## 3 Requirements
**Functional**
- F1 Generator: `build(&ModulusParams{p,n}) -> Vec<Op>`. Track G generator reads p/n only through it.
- F2 Harness `analyze_modulus(&ModulusParams) -> Option<FoldInfo>`: Some iff F=2^n-p >=0, bitlen<=B_f, signed-digit weight<=W_f (documented consts).
- F3 Ranked (n=256): `secp256k1` (on-curve), `p256`, `hidden_sparse_256`, `hidden_dense_256`.
- F4 Probe (unranked, reported only): public `p192`,`p224`,`p384` + `hidden_192`,`hidden_224`,`hidden_384`.
- F5 Hidden primes from an epoch seed unavailable before window close (§5.1).
- F6 Per instance draw N_i fresh inputs independent of circuit and contestant-visible data. Count failures in union cls∪anc∪pha.
- F7 eps_UB = one-sided 95% Clopper-Pearson (§5.3).
- F8 `S_adj=Q*T/(1-eps_UB)`. Instance fails gate if eps_UB>eps_max.
- F9 GScore = geomean S_adj over the 4 ranked. Any ranked gate fail -> not Track G eligible (still Track S).
- F10 Report gap = GScore / S_adj(secp256k1).
- F11 Track S output bit-identical to pre-change.
- F12 secp256k1: on-curve points. Siblings: algebraic-domain inputs (§5.2), pending OQ3.

Modularity: F6-F8 (error pricing on an independent corpus) are separable from F3-F5, F9. Implementing them first on the single secp256k1 instance is valid.

**Integrity**
- S1 No path derives hidden primes/fresh inputs from pre-close data.
- S2 Domain-separate by epoch.
- S3 Generator gets only `ModulusParams` + shared helpers (no instance id, hidden flag, epoch).
- S4 Hidden primes/seeds absent from contestant-visible output before reveal.
- S5 Deterministic: byte-identical reruns; evaluation order irrelevant.

**Non-functional**: N1 no new crates (§8). N2 Track S not slowed; Track G only on request/finalization. N3 runtime unmeasured: benchmark (T9) before fixing N_i. N4 unit tests for all new logic.

## 4 Formats
Manifest `family/<epoch>.json`: `{epoch, ranked:[{id,n,source:"public"|"derived",curve_points:bool}], probe:[same], n_fresh:{ranked,probe}, eps_max, domain:"quantum_ecc-trackg-v1"}`. Placeholders pending T0/T9: n_fresh ranked 10000, probe 3000; eps_max 0.005. Public primes defined in code from standards, not in manifest.
Result `results/<epoch>/<submission>.json`: `{submission, track_g_eligible, instances:[{id,q,t,fails,n,eps_ub,s_adj,gate}], gscore, gap, probe:[{id,gate:pass|fail|build_error|build_timeout,s_adj}]}`. Hidden primes absent until reveal.

## 5 Algorithms
**5.1 Hidden primes.** `seed_h = SHAKE256(u32le(len(domain)) || domain || u64le(epoch) || seed_bytes)`; absorb `"inst/"+id` first per instance. Sparse: odd c, 3<=c, bitlen(c)<=40, p=2^n-c, retry until prime. Dense: n random bits, set top and bottom bit, retry until prime. Primality: Miller-Rabin >=40 rounds, bases from the stream; require p odd and p>2^(n-1). Candidate sequence depends only on the stream.

**5.2 Inputs.** Three disjoint sets from independent XOF labels: cost corpus (same distribution as existing scoring `[A]`), fresh (N_i), corner block per instance: xR-xA in {1,2,p-1,p-2,small,~p/2}; coords in {0,1,2,p-2,p-1}; values at every 64-bit limb boundary of n; alternating-bit masks.
- secp256k1: on-curve; filter identity and xR=xA as existing evaluator `[A]`.
- Siblings: random (xR,yR,xA,yA) in F_p^4 with xR≠xA and xR'≠xA. Expected: λ=(yR-yA)(xR-xA)^-1, xR'=λ²-xR-xA, yR'=λ(xA-xR')-yA. Reference implementation MUST be independent of the generator under test.

**5.3 Failures and bound.** Trial fails if any of: classical mismatch, nonzero scratch qubit, nonzero phase parity `[A: cls/anc/pha]`; count each trial once. eps_UB (α=0.05): f=0 -> 1-α^(1/n); f>0 -> smallest ε with P(Bin(n,ε)<=f)=α via bisection on log-space binomial CDF. No crates.

**5.4 Build errors.** Panic caught -> `build_error`. Per-instance limit 60 s (placeholder) -> `build_timeout`.

## 6 Interfaces
CLI `[A]`: `evaluate --src` (Track S, unchanged); `trackg --src --family public`; `trackg --src --epoch E --final`; `audit --src --family public` (T0 taxonomy).
Exit 0 only on full success; else first stderr line is one of: `TRACKG_BUILD_ERROR`, `TRACKG_BUILD_TIMEOUT`, `TRACKG_GATE_FAIL`, `TRACKG_BAD_MODULUS` (even/wrong width/not prime), `TRACKG_SEED_MISSING`, `TRACKG_TOO_EARLY`.

## 7 Acceptance tests
| ID | Test -> expected |
|----|------------------|
| A1 | analyze_modulus(secp256k1) -> Some, F=2^32+977, digits reconstruct F |
| A2 | dense random 256-bit prime -> None |
| A3 | P-256 -> pin actual behavior |
| A4 | each public prime: odd, correct width, MR 64 rounds, equals standard's formula; quote output |
| A5-7 | same seed twice -> same primes; seed differing 1 bit -> different; epochs 7 vs 8 -> different |
| A8 | Clopper-Pearson f=0,n=1000 -> 0.0029912 (±1e-6); f=2,n=1000 -> matches independent reference (state method) |
| A9 | reference chord impl == independent impl (e.g. Python `pow(x,-1,p)`) on 10k random inputs/instance |
| A10 | Track S: pre-change vs new harness on same circuit -> bit-identical |
| A11 | control: generic, unoptimized, correct for any odd p -> passes gate on all ranked |
| A12 | control: literal n=256 budget -> passes ranked, fails probe widths (reported) |
| A13 | control: `if p==SECP256K1 {fast} else {generic}` -> no GScore benefit from fast path on other 3; report numbers |
| A14 | control: dispatch on secp256k1+P-256 -> hidden instances expose generic cost; report GScore with/without hidden |
| A15 | build panic -> caught, `build_error`, harness continues |
| A16 | same (p,n) twice -> identical build output (S3) |
| A17 | `trackg --final` twice -> byte-identical |
| A18 | non-final run output/logs contain no hidden primes/seeds |
| A19 | final run succeeds offline |

A11-A14 are the key regressions. If a control cannot be built, state why and use a synthetic oracle. Do not claim a pass otherwise.

## 8 Tasks and constraints
- T0 **Audit (no Track G implementation):** minimally feed alternative p to current top generator(s); run public siblings; write `docs/trackg_audit.md` per instance: built?, passed?, eps, failure locus (reduction cell / GCD budget / bound violation), mechanical (literal) vs structural (assumes sparse form). Informs eps_max (OQ1).
- T1 mapping doc. T2 params + analyze_modulus (A1-3). T3 public instances (A4). T4 hidden derivation + primality (A5-7). T5 sampling + reference chord (A9). T6 failure counter + CP (A8). T7 eval loop/scoring/results (A10, A15-19). T8 controls (A11-14). T9 benchmark seconds/instance at n=192,224,256,384; recommend N_i, eps_max from T0 rates. T10 contestant docs.
- Deps: T2,T3<-T1; T4<-T2; T5<-T3; T6<-T5; T7<-T4,T6; T8<-T7; T9<-T7,T0; T10<-T7.
- Constraints: no new deps (bigint/primality from existing Cargo.toml; if harness is fixed 256-bit, STOP and ask). Forbidden: changing Track S; giving the generator more than `ModulusParams`+helpers; network in evaluation; hash-map order in hashed/serialized/ranked data; unverified primes; weakening existing invariants.

## 9 Open questions (escalate, do not decide)
- OQ1 eps_max: best circuit ~0.19% any-channel on 100k independent inputs; references <=2^-13.3 (~0.010%). Decide after T0.
- OQ2 Conflict with `AGENT_SPEC_commit_reveal_tier3.md` F6 (any failure over 100k disqualifies; would disqualify current top circuits). Amend to the eps_UB rule before adopting both. Do not implement both as written.
- OQ3 Algebraic-domain sibling inputs miss circuits exploiting the curve equation. Accept, or require on-curve points (needs sqrt mod p)?
- OQ4 Number of hidden ranked members (2 now).
- OQ5 Rank n=384? Needs normalization or a frozen reference generator.
- OQ6 Fix analyze_modulus thresholds in the rules?
- OQ7 Seed mechanism and custody.
- OQ8 Are top circuits source-generated (`src/point_add/`) or raw op streams (cannot enter Track G)?

## 10 Done
All §7 tests pass (skips listed with reason); T0 report exists; A10 bit-identical; `cargo build`/`cargo test` clean, `Cargo.toml` diff empty or justified; A11-A14 show dispatch and literal budgets do not pay; T9 numbers measured, not estimated; unresolved §9 items listed in the final report.
