# Context for Claude Code: QCB ecdsa.fail 2.0 repo

This repo holds research for QCB's ecdsa.fail 2.0 project: claims, replies, audits and notes about the Layr-Labs ecdsa.fail challenge (quantum point-addition circuits for secp256k1, scored S = Q × T). The goal is a benchmark harness whose winners generalize and are usable in Shor's algorithm. Start with `README.md` (layout, conventions) and `notes/research-notes.md` (facts and numbers).

## Rules

1. **Evidence labels.** Label every number measured, source-read, claimed or inferred, and cite `path:line` plus the commit for code facts. Do not upgrade a label without new evidence.
2. **Teammates' files.** `context/teammate-claims/` holds other people's work. Never edit it.
3. **Versioning.** Bump the version in the filename (`claim_v0.2.md`) instead of overwriting a posted version, and add an entry to `LOG.md`.
4. **Untrusted code.** Contestant code from the challenge repo is untrusted. Do not build or run it outside a sandbox, and ask before any build.
5. **Two parts per claim or reply.** Each one has a short Discord summary (`discord_post.md`: Discord markdown, no LaTeX, escape underscores) and an agent-readable doc. The doc has a TL;DR/status section with evidence labels, findings, proposed requirements (MUST/SHOULD), verification tasks, open questions, and sources.
6. **Live numbers.** Leaderboard numbers change weekly. Re-check before reusing anything from `notes/`.

## Where things are

- Challenge clone: expected at `../ecdsafail-challenge` (not part of this repo).
- Audit workflow: copy an `agent_brief.md` into the challenge clone as `CLAUDE.md` and work in `qcb_audit/` there. Bring reports back to `claims/<name>/audit/`.
