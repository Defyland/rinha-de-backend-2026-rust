# Vectorization Proof

This repository now includes executable proof for the two worked examples from the official Rinha 2026 fraud-detection rules.

What is proved:

- the legit example vector matches the published 14-dimension shape
- the fraud example vector matches the published 14-dimension shape
- the implementation keeps the `-1` sentinel for missing `last_transaction`

What is not claimed here:

- final challenge accuracy against the 3-million-reference dataset
- submission-grade p99 under the official resource envelope

Why this proof matters:

- it validates the contract before any ANN or memory-optimization work
- it gives reviewers a concrete place to compare code against the official rules

Run:

```bash
cargo test --all-targets engine::tests::vectorizes
```
