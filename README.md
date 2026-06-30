# Rinha de Backend 2026 Rust

[![CI](https://github.com/Defyland/rinha-de-backend-2026-rust/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Defyland/rinha-de-backend-2026-rust/actions/workflows/ci.yml)

Rust reference slice for the [Rinha de Backend 2026](https://github.com/zanfranceschi/rinha-de-backend-2026) fraud-scoring challenge.

This repository now implements the real contract instead of a bootstrap:

- `GET /ready` on port `9999`
- `POST /fraud-score` with the official 14-dimension vectorization rules
- exact k-NN (`k=5`) scoring over a local reference index
- challenge-shaped `docker-compose.yml` with one load balancer and two API instances
- CI, tests, ADRs, architecture notes, and runnable resource-loading commands

It is intentionally an **exact baseline**, not a final competition submission. The current engine optimizes for correctness and public reviewability first. The README is explicit about the remaining memory and p99 work needed before this would be competitive under the official `1 CPU / 350 MB` envelope.

## What this project is for

The challenge asks for a fraud API backed by vector search over 3 million labeled references. This repo demonstrates the systems slice that matters first:

- deterministic vectorization from the official payload contract
- exact neighbor search and decision threshold behavior
- resource loading that works with bundled sample data or the official `references.json.gz`
- a topology that mirrors the competition requirement enough to evaluate locally

## How it behaves

1. Startup loads `normalization.json`, `mcc_risk.json`, and a reference dataset.
2. `POST /fraud-score` converts the request into the official 14-dimension vector.
3. The engine finds the 5 nearest neighbors with Euclidean distance.
4. `fraud_score` is `fraud_neighbors / 5`.
5. `approved` is `fraud_score < 0.6`.

## Architecture

- `src/config.rs`: runtime bind/resource configuration
- `src/model.rs`: API payloads and reference-record schema
- `src/engine.rs`: vectorization, resource loading, and exact k-NN scoring
- `src/api.rs`: Axum HTTP surface

Supporting docs:

- [docs/architecture/overview.md](docs/architecture/overview.md)
- [docs/adr/0001-exact-knn-baseline.md](docs/adr/0001-exact-knn-baseline.md)
- [docs/verification/vectorization-proof.md](docs/verification/vectorization-proof.md)

## How to run in 5 minutes

1. Run the API against the bundled sample references:

```bash
cargo run
```

2. Check readiness:

```bash
curl -s http://127.0.0.1:9999/ready
```

3. Score the first official example payload:

```bash
curl -s http://127.0.0.1:9999/fraud-score \
  -H 'content-type: application/json' \
  --data @docs/examples/legit-request.json
```

4. Run the full verification loop:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

5. Run the load-balanced local topology:

```bash
docker compose up --build
```

## Official resources

This repo vendors the small, review-friendly assets:

- `resources/normalization.json`
- `resources/mcc_risk.json`
- `resources/example-references.json`

To download the official 3-million-reference dataset:

```bash
./scripts/fetch-official-resources.sh
```

Then run the server against it:

```bash
RINHA_REFERENCES_PATH=resources/references.json.gz cargo run
```

## API contract

The local OpenAPI mirror lives in [openapi.yaml](openapi.yaml).

`GET /ready`
- returns `200` once resources are loaded

`POST /fraud-score`
- request matches the official challenge payload
- response body:

```json
{
  "approved": true,
  "fraud_score": 0.2
}
```

## Evaluation posture

What already works:

- correct 14-dimension vectorization from the published rules
- exact `k=5` scoring logic
- gzip or plain JSON resource loading
- HTTP contract coverage for readiness and scoring
- local challenge-shaped topology with Nginx round-robin

What is intentionally still not final:

- exact search over the full 3-million dataset is memory-heavy and not yet budgeted for two API containers inside the official limit
- no ANN index, quantization, mmap compaction, or shared-cache strategy yet
- no benchmark capture or challenge submission branch yet

## Key trade-offs

- **Correctness first**: exact search keeps the baseline easy to verify against the official rules.
- **Small repo surface**: the sample dataset is bundled; the full dataset is fetched on demand.
- **Honest deployment story**: `docker-compose.yml` proves topology, but the current exact baseline should be treated as a correctness/reference build, not a score-maximizing submission.

## Files reviewers should read first

- [docs/architecture/overview.md](docs/architecture/overview.md)
- [docs/adr/0001-exact-knn-baseline.md](docs/adr/0001-exact-knn-baseline.md)
- [docs/verification/vectorization-proof.md](docs/verification/vectorization-proof.md)
- [docs/examples/legit-request.json](docs/examples/legit-request.json)
- [docs/examples/fraud-request.json](docs/examples/fraud-request.json)

## License

This repository is published under the MIT License. See
[LICENSE.txt](LICENSE.txt).

That keeps the exact-baseline implementation and its verification notes
reusable for study and internal experimentation.
