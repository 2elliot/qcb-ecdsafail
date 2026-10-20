# taint_count

The instrument behind the classical-addend audit (`../audit_report.md`). It is a standalone Rust crate built on the challenge's harness library (`quantum_ecc`). It contains no contestant code and adds no new crates.

## What it does

- **Static pass.**
  - Taints every classical bit as offset-derived (OFF), measurement-derived (MEAS), both, or constant.
  - Classifies every CCX/CCZ by the taint of its condition and of any classically-known control.
  - Lists the offset-conditioned X load/unload runs, and the Toffolis between them.
- **Dynamic pass.**
  - Replays the official 9,024 inputs and reports executed Toffolis per taint class.
  - Cross-checks every batch against `quantum_ecc::sim::Simulator`.
- **`--static`** runs only the static pass, which is much faster.

## Building and running

`Cargo.toml` points at the harness with `quantum_ecc = { path = "../.." }`, so the crate only builds from inside a challenge clone at `<clone>/qcb_audit/taint_count/`. Copy it there first:

```
cargo build --release --locked --offline
target/release/qcb_taint_count path/to/ops.bin            # static + dynamic
target/release/qcb_taint_count path/to/ops.bin --static   # static only
```

The `ops.bin` path must be the first argument. Only run it on op streams you built in a sandbox, because building a submission runs contestant code.

## Results

Raw outputs live in `../results/`:

| File | What it shows |
| --- | --- |
| `v2_60d61859_official_eval.txt`, `v2_incumbent_60d61859_taint.txt` | Incumbent 8e9c9a22: official score reproduced; 0 offset-conditioned Toffolis |
| `v2_3616dbf2_897dda2b_official_eval.txt`, `v2_3616dbf2_897dda2b_taint.txt` | Ping-pong GCD 3616dbf2: same |
| `p1_3493c837_31f9c58c_official_eval.txt`, `p1_3493c837_31f9c58c_taint.txt` | Lowest-Q promoted 3493c837: same |
| `followup_offset_by_kind.txt` | Every op kind executed under an offset-derived condition (X, bit ops, push/pop only) |
| `p2_add3x_vs_orig.txt` | +433.4 T to compute 3·x_A in-circuit (source: `../add3x_bench/qcb_add3x.rs`) |
| `v3_pr15_qrom_cost.txt` | Lookup cost at w = 16: 65,534 T with measurement-based uncompute |
| `followup_lowest_q_promoted.txt` | Lowest-Q promoted submissions, from bot scores |
| `v4_top3_metadata.txt` | Current top-3 S/Q/T from PR metadata (claims and bot-recorded) |
