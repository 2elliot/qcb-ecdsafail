# Pending reply drafts (as of 2026-10-22; status updated 2026-10-29)

Five of these six are unposted; the P003 reply was posted on Oct 20. The plan is to wait for more teammates to respond, then craft final replies from these drafts. Each draft records:

- what the other post says;
- what we re-checked;
- the points to make;
- a paste-ready Discord version as of today.

Re-check every number against any newer posts before sending.

## Status

| Draft | Thread | Responding to | Status |
| --- | --- | --- | --- |
| `p003_error_pricing.md` | P003 (per-run error pricing) | Mike (P003) | **Posted Oct 20** (earlier version). Aodhfionn replied to it on Oct 23 |
| `track_q.md` | My classical-addend claim | Track Q synthesis | Drafted; detailed doc and claim v0.2 exist as Claude Docs (links inside) |
| `r003_cmod.md` | C-MOD / Track G | Mike's R003 | Drafted |
| `trackg_synthesis.md` | C-MOD / Track G | Joel's Track G synthesis | Drafted |
| `r002_ctail.md` | C-TAIL / Fiat-Shamir | Mike's R002 (PFS-priced) | Drafted |
| `assurance_architecture.md` | C-TAIL / Fiat-Shamir | Joel's Shor circuit assurance architecture | Drafted |

## Positions held across all threads

Keep replies consistent with these, and update this list if a position changes.

1. **k = 28, not 32.** Google's whitepaper (Appendix A.3) and the trapped-ion estimate use 28 windowed additions at w = 16. 32 = 2·256/16 is an inferred count. Joel's assurance doc agrees, and recommends k as a wrapper-profile parameter.
2. **(1 − ε)^−k is a diagnostic surrogate, not physics.** It's fine as a pricing rule if labeled. If anything it is lenient: the trapped-ion bound E[P] ≥ (√P₀ − 2p_f)² falls about 4/√P₀ times faster for small errors.
3. **Public corpus + classical addend is broken.** Inferred from audit V1 semantics, not built yet:
    - Classical bit ops are free, stacked conditions AND, and condition-false Toffolis aren't counted. So a circuit can recognize any public test input for free.
    - It can switch off all arithmetic on public cost shots and XOR in precomputed answers, so T̄_cost ≈ 0 and the score ≈ 0.
    - It can patch every failing public error-corpus input for 0 Toffolis.
    - Today's hash-derived inputs are immune only because the corpus depends on the circuit. This breaks C-TAIL's fixed public seed, R002's PFS-priced design, and P003 on a fixed corpus.
4. **The quantum addend boundary is necessary for any public corpus.** Track Q's typed boundary, or the address-indexed `Lookup` mode from claim v0.2, makes recognition cost Toffolis on every shot. Quantum-controlled gates are charged whether or not they fire. With a quantum addend, public-corpus patching costs ~56 T per failing point and stops paying once N_err ≳ 4 × 10⁵.
5. **Score noise is the next problem after verdict stability.** At N = 10,000 and k ≈ 30, an approximate circuit's priced score moves ~±1.3–1.4% per draw, ~90× the median October step (0.016%). Fixes: adaptive N for f > 0, or one shared secret corpus per epoch so near-identical circuits compare almost exactly.
6. **Specialization to secp256k1's prime is legitimate.** Track G is a generality track, not a replacement ranking. Any soft floor must not clamp honest specialization.
7. **No default switch before:** a passing reference circuit in the new mode, a verifier seed not derived from circuit bytes, and published constants.
8. **Disclose our own holes.** R1's free output patching and R3's idle anchor are both acknowledged in claim v0.2.

## New since 2026-10-22 (from `notes/verified-claims-catalogue-2026-10-29.md`)

These are recorded as information for crafting replies; no positions above have been changed.

- **Leader moved** to 72ccdb8 (923.5M = 743,575 T × 1,242 Q). Its errors on 128k independent inputs are 0.200% wrong output, 0.036% phase-only, and no dirty ancillas (Aodhfionn). Drafts that cite 7499c90, 54fc4f25 or 939.1M need updated numbers.
- **Aodhfionn on our claim (thread 3)** (`R_classical_addend_offset_bits__AGENT.md`):
    - 5,120 offset-conditioned ops on HEAD, all Clifford/Pauli and 0 Toffolis, so 0 T discount today;
    - suggested rule wording "never a CCX/CCZ";
    - S_W "double-prices width";
    - independently raises offset-keyed patching of a fixed public corpus.
    - Relevant to `track_q.md` and to claim v0.2.
- **Aodhfionn on P003, and his new thread C-AMP** (`R_P003_amplitude_exponent__AGENT.md`, `C-AMP_…partial.md`): the error-pricing exponent should be (1 − ε)^(2k) for wrong/ancilla and (1 − 2ε)^(2k) for phase, supported by exact simulation. He names our P003 reply among the targets.
    - Bears on position 2 and on every draft that uses (1 − ε)^−k pricing: `r003_cmod.md`, `trackg_synthesis.md`, `r002_ctail.md`, `assurance_architecture.md`.
- **Jack's new thread C-03 and Yara's reply** (`C-03-…partial.md`, `Antigravity_…partial.md`): R/HMR consume 8 XOF bytes each, so clean-reset padding can steer a dirty reset's recorded phase under a fixed seed. Yara argues the Fiat-Shamir binding prevents this in today's harness.
    - Relates to F6 of our Fiat-Shamir reply, and to R002's dirty-reset flagging.
- **Not yet answered by us:** C-AMP, C-03, and Aodhfionn's two replies, in addition to the five unposted drafts.

## Before posting anything

- [ ] Build the zero-cost public-corpus demo (`notes/next-steps.md` item 0). It turns position 3 from "inferred" into "measured" and is the strongest point in four of the six drafts.
- [ ] Re-read each thread for newer replies; update numbers (leader is now 72ccdb8; k; N) and drop points someone else already made (Aodhfionn already raised offset-keyed patching).
- [ ] Decide whether to fold position 3 into claim v0.2 (as v0.3) and into the Fiat-Shamir reply (v0.3).
