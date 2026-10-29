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
│       ├── claim_v0.1.md         agent-readable claim doc (v0.2 is a Claude Doc; see links below)
│       ├── discord_post.md       paste-ready Discord summary + thread title
│       └── audit/
│           ├── audit_report.md   full Claude Code audit (V1–V4, follow-ups, critique)
│           ├── agent_brief.md    the CLAUDE.md brief the audit ran under
│           ├── add3x_bench/      source of the +433.4 T measurement (qcb_add3x.rs)
│           ├── results/          raw outputs of every audit run (11 .txt files)
│           └── taint_count/      the audit instrument (builds only inside a challenge clone)
├── replies/
│   ├── fiat-shamir/              posted reply to "Eliminating Fiat-Shamir tail-nonce grinding"
│   │   ├── reply_v0.2.md
│   │   └── discord_post.md       full reply + follow-up text
│   ├── track-g/                  posted reply to the modulus-specialization / Track G thread
│   │   ├── reply_v0.2.md
│   │   └── discord_post.md
│   └── drafts/                   unposted replies (held until threads settle)
│       ├── README.md             status table + positions held across all threads
│       └── *.md                  one draft per pending reply (P003, Track Q, R003, Track G synthesis, R002, assurance architecture)
├── context/
│   └── teammate-claims/          teammates' agent docs this work responds to (read-only), plus summaries.md
└── notes/
    ├── research-notes.md         durable facts, numbers, formulas, sources
    ├── verified-claims-catalogue-2026-10-29.md  every thread, message and attachment in #verified-claims
    └── next-steps.md             follow-ups (item 0: zero-cost public-corpus demo), open questions, future claims
```

## Status (as of 2026-10-29)

| Item | Status |
| --- | --- |
| Claim: classical-addend surface | v0.1 posted. v0.2 (quantum boundary, charged lookups; responds to Track Q) drafted as a Claude Doc |
| Reply: Fiat-Shamir thread | v0.2 posted |
| Reply: Track G thread | v0.2 posted |
| Replies to new posts | P003 reply posted Oct 20; five drafts in `replies/drafts/` held. Also unanswered: C-AMP, C-03, and Aodhfionn's two replies (see the catalogue) |
| Biggest open item | Zero-cost public-corpus demo (`notes/next-steps.md` item 0) |

## Conventions

- **Evidence labels.** Every number is **measured**, **source-read**, **claimed** (by a submitter) or **inferred**. Never upgrade a label without new evidence.
- **Code references** use `path:line` plus the commit, e.g. `sim.rs:82-87` at `3dee0cd6`.
- **Versions** live in filenames (`claim_v0.1.md`, `reply_v0.2.md`). Post a new version rather than silently editing a posted one.
- **Safety.** Contestant code is untrusted. Build and run it only in a sandbox (WSL2, a container or a VM). Identify circuits from metadata (`results.tsv`, PR bodies) before building anything.
- **Teammates' files** in `context/` are kept as received. Don't edit them; respond in your own files.
- **Consistency.** Before posting a reply, check it against the positions list in `replies/drafts/README.md`.

## Working with agents

1. Clone the challenge next to this repo: `git clone https://github.com/Layr-Labs/ecdsafail-challenge ../ecdsafail-challenge`.
2. For an audit, copy a brief like `claims/classical-addend/audit/agent_brief.md` into that clone as `CLAUDE.md`. Put context files in `qcb_audit/context/` and have the agent write to `qcb_audit/`.
3. Bring the agent's report back into `claims/<name>/audit/` here.
4. When you work in this repo itself, the root `CLAUDE.md` gives Claude Code the project context.

## Editable originals

These docs were written as Claude Docs, the editable originals. The `.md` files here are snapshots; after editing a doc, re-export it (doc name → Export → Markdown) and replace the file.

- Claim v0.1: https://claude.ai/code/artifact/fe764c76-42c7-4b9c-8bc1-89a4296a7c92
- Claim v0.2 (not yet exported or posted): https://claude.ai/artifact/WMMnkjnNxDvg9L96cjvQUV
- Reply to Track Q (not yet exported or posted): https://claude.ai/artifact/1EbBt5SawK6wK6npcmK9ba
- Fiat-Shamir reply: https://claude.ai/code/artifact/1ae632bf-60d2-4074-ad63-9216cd1e1934
- Track G reply: https://claude.ai/code/artifact/51966fa1-a181-47f7-9e1d-483ac0935f40

Keep this repo **private**. It contains teammates' unpublished specs and internal Discord posts.
