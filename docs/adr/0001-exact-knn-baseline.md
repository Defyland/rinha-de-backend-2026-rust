# ADR 0001: Start With An Exact k-NN Baseline

## Context

The repo started as a bootstrap with no executable API, no scoring engine, and no evidence that the official Rinha 2026 contract was understood correctly.

The first public-ready loop needed to prove:

- the API shape
- the vectorization rules
- the neighbor-selection logic
- the local topology requirement

## Options considered

1. Implement ANN/HNSW immediately.
2. Implement an exact baseline first, then optimize.
3. Stop at docs and defer all code.

## Chosen option

Option 2: implement an exact baseline first.

## Pros

- Easy to verify against the official examples.
- Small enough to review in one sitting.
- Creates a stable correctness target before optimization.
- Lets future benchmark work isolate performance changes from behavior changes.

## Cons

- Full-dataset exact search is too expensive to treat as submission-ready.
- Two API instances holding large in-memory indexes will hit the competition resource budget quickly.

## Consequences

- The repo is now truthful and runnable.
- Reviewers can validate the core fraud-decision contract without reverse-engineering a premature ANN layer.
- The next loop should focus on compact indexing and measured p99 improvements, not on redefining the API or vector math.

## Verification evidence

- `cargo test --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- `docker compose config`
