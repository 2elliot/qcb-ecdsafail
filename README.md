# QCB ecdsa.fail 2.0: working repo

Research notes, claims and replies for QCB's ecdsa.fail 2.0 project. The project is designing a benchmark harness for our own version of the [ecdsa.fail](https://github.com/Layr-Labs/ecdsafail-challenge) challenge. The goal is that winning point-addition circuits generalize and remain usable as components of Shor's algorithm on real hardware, instead of being tuned to quirks of the current harness.

Each week the team posts **verified claims** in Discord: an exploit in the current challenge, or a fix for one. Each claim or reply has two parts. A short human summary goes in the Discord post. A detailed, agent-readable doc goes in the attachment, so teammates' agents can check it.

## Layout

```
.
├── README.md                     this file
├── CLAUDE.md                     context for Claude Code when working in this repo
├── LOG.md                        week-by-week log of what was done and decided
├── claims/
│   └── classical-addend/         my claim (week of 2026-10-19)
│       ├── claim_v0.1.md         agent-readable claim doc
│       ├── discord_post.md       paste-ready Discord summary + thread title
│       └── audit/
│           ├── audit_report.md   full Claude Code audit (V1–V4, follow-ups, critique)
│           ├── agent_brief.md    the CLAUDE.md brief the audit ran under
│           ├── add3x_bench/      source of the +433.4 T measurement (qcb_add3x.rs)
│           ├── results/          raw outputs of every audit run (11 .txt files)
│           └── taint_count/      the audit instrument (builds only inside a challenge clone)
├── replies/
│   ├── fiat-shamir/              reply to "Eliminating Fiat-Shamir tail-nonce grinding"
│   │   ├── reply_v0.2.md
│   │   └── discord_post.md       full reply + follow-up text
│   └── track-g/                  reply to the modulus-specialization / Track G thread
│       ├── reply_v0.2.md
│       └── discord_post.md
├── context/
│   └── teammate-claims/          teammates' agent docs this work responds to (read-only)
└── notes/
    ├── research-notes.md         durable facts, numbers, formulas, sources
    └── next-steps.md             follow-ups, open questions, candidate future claims
```

## Status (as of 2026-10-19)

| Item | Status |
| --- | --- |
| Claim: classical-addend surface (restrict offset bits to loads) | v0.1 drafted, backed by the audit |
| Reply: Fiat-Shamir thread | v0.2 (F4 corrected after the audit) |
| Reply: Track G thread | v0.2 (audit evidence added) |
| Open measurements | Leader's k and division-seam width (AT2/AT3), incumbent fresh-input failure rate; see `notes/next-steps.md` |

## Conventions

- **Evidence labels.** Every number is **measured**, **source-read**, **claimed** (by a submitter) or **inferred**. Never upgrade a label without new evidence.
- **Code references** use `path:line` plus the commit, e.g. `sim.rs:82-87` at `3dee0cd6`.
- **Versions** live in filenames (`claim_v0.1.md`, `reply_v0.2.md`). Post a new version rather than silently editing a posted one.
- **Safety.** Contestant code is untrusted. Build and run it only in a sandbox (WSL2, a container or a VM). Identify circuits from metadata (`results.tsv`, PR bodies) before building anything.
- **Teammates' files** in `context/` are kept as received. Don't edit them; respond in your own files.

## Working with agents

1. Clone the challenge next to this repo: `git clone https://github.com/Layr-Labs/ecdsafail-challenge ../ecdsafail-challenge`.
2. For an audit, copy a brief like `claims/classical-addend/audit/agent_brief.md` into that clone as `CLAUDE.md`. Put context files in `qcb_audit/context/` and have the agent write to `qcb_audit/`.
3. Bring the agent's report back into `claims/<name>/audit/` here.
4. When you work in this repo itself, the root `CLAUDE.md` gives Claude Code the project context.

## Editable originals

The three docs were written as Claude Docs, the editable originals. The `.md` files here are snapshots. If you edit a doc, re-export it (doc name → Export → Markdown) and replace the file.

- Claim: https://claude.ai/code/artifact/fe764c76-42c7-4b9c-8bc1-89a4296a7c92
- Fiat-Shamir reply: https://claude.ai/code/artifact/1ae632bf-60d2-4074-ad63-9216cd1e1934
- Track G reply: https://claude.ai/code/artifact/51966fa1-a181-47f7-9e1d-483ac0935f40

Keep this repo **private**. It contains teammates' unpublished specs and internal Discord posts.
