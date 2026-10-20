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

**Open:** see `notes/next-steps.md`.
