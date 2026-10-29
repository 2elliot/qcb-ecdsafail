# Log

## Week of 2026-10-19: first verified-claims assignment

**Assignment:** post at least 1 thread in the verified-claims channel (an exploit or a fix) and at least 2 replies to others' threads. Each needs a human summary plus a detailed agent doc.

**Read and replied to:**
- *Eliminating Fiat-Shamir tail-nonce grinding* (fixed public test set, canonicalizer, exact checks). Reply: fixed public points are patchable; the "15%" figure is impossible; the canonicalizer is redundant; forward∘reverse is not implemented at HEAD (a correction made in v0.2); measurement randomness must be secret; use offset masks instead of complete adders. → `replies/fiat-shamir/`
- *Modulus specialization / Track G*. Reply: compound error over K = 28 calls; size n_fresh to about 30k and tie ε_max to attack success; the generator must be pure (no env/files); draw the corner block fresh; P-256 threshold split. → `replies/track-g/`

**Claim:** started from the hypothesis that the classical addend hides a ~188k-Toffoli discount relative to windowed Shor. A Claude Code audit refuted that:
- the older leaders have 0 addend-conditioned Toffolis, and the specialization bound is ≈0.75k T;
- the paper's +29.6% is lookup cost.

The audit also found that the current top lineage has started conditioning arithmetic on offset bits. The claim was reframed as an emerging exploit plus a fix: restrict offset bits to loads, and report a windowed-adjusted score. → `claims/classical-addend/`

**Decisions and lessons:** check metadata before building; sandbox contestant code; a negative result was turned into an exploit-plus-fix claim once V4 found live use.

## 2026-10-20 to 10-22: repo, responses, drafts

- **Repo.** Created the private repo and linked it to the Claude project.
- **New posts from teammates** (docs and summaries in `context/teammate-claims/`):
    - P003 (per-run error pricing);
    - Track Q synthesis (a response to my claim);
    - Mike's R003 (to C-MOD) and R002 (PFS-priced, to C-TAIL);
    - Joel's Track G synthesis and Shor circuit assurance architecture.
- **My response to Track Q.** Adopt its typed quantum boundary; withdraw R1/R3; propose an address-indexed mode with a charged `Lookup` macro-op. Drafted as Claude Docs: a reply to Track Q and **claim v0.2** (links in `replies/drafts/track_q.md`).
- **New finding (inferred, not built).** Under the classical-addend ABI, any public test corpus can be recognized for free: cost-corpus T̄ → 0 and error-corpus failures → 0. This affects C-TAIL, PFS-priced and P003-on-a-fixed-corpus. It makes the quantum boundary necessary for public corpora. The demo is `notes/next-steps.md` item 0.
- **Decision.** Hold all six replies until more teammates respond; drafts and cross-thread positions are in `replies/drafts/`.

**Open:** see `notes/next-steps.md`.

## 2026-10-29: #verified-claims catalogue

- **Catalogue.** Read all six threads in #verified-claims through the browser (read-only) and catalogued them in `notes/verified-claims-catalogue-2026-10-29.md`.
- **Correction.** The P003 reply had already been posted on Oct 20, though the drafts index had listed it as unposted.
- **New since Oct 22:**
    - Aodhfionn Downs replied to our claim (offset ops on HEAD are 0 Toffolis) and to P003 (amplitude exponent);
    - Aodhfionn opened C-AMP (error pricing off by 2× in the exponent, 4× for phase);
    - Jack opened C-03 (measurement-outcome steering), and Yara replied to it;
    - the leader moved to 72ccdb8 (923.5M).
- **Repo additions.** Their agent docs are in `context/teammate-claims/`: two captured in full, three as partial captures to replace with the originals.
