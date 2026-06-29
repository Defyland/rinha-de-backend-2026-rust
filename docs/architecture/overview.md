# Architecture Overview

## Context

The official challenge requires:

- `GET /ready`
- `POST /fraud-score`
- vectorization with 14 published dimensions
- nearest-neighbor search against 3 million labeled reference vectors
- a load balancer and two API instances on port `9999`

This repository implements the smallest complete slice that proves the behavior honestly.

## Current runtime shape

```text
client
  -> nginx load balancer (:9999)
      -> api-1 (Rust/Axum)
      -> api-2 (Rust/Axum)
```

Each API instance loads:

- normalization constants
- MCC risk lookup
- a reference dataset from JSON or gzip-compressed JSON

## Request flow

1. Axum accepts `POST /fraud-score`.
2. The payload is transformed into the official 14-dimension vector.
3. The engine scans the reference dataset and keeps the 5 closest neighbors by Euclidean distance.
4. The engine returns:
   - `fraud_score = fraud_neighbors / 5`
   - `approved = fraud_score < 0.6`

## Why exact search first

Exact k-NN is deliberately simple:

- it matches the official scoring behavior directly
- it is easy to test from the published examples
- it creates a truthful baseline before adding approximate indexes or binary compaction

## Operational limits

This design is correct but not yet final for the competition envelope:

- loading the full 3-million dataset naively in both API containers will stress the memory budget
- brute-force exact search is unlikely to be competitive on p99 without further indexing work

That makes the next improvement loop clear:

1. preprocess the official references into a compact binary layout
2. evaluate mmap or quantized ANN/exact hybrid approaches
3. capture real benchmark evidence before any submission claims
