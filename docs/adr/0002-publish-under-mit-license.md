# ADR 0002: Publish the Repository Under the MIT License

## Context

This repository is already a public, runnable reference implementation for the
Rinha 2026 fraud-scoring contract. Without an explicit license, reviewers can
inspect the exact baseline but still lack a clear reuse boundary for study or
experimentation.

## Options considered

1. Keep the default all-rights-reserved posture.
2. Publish under the MIT License.
3. Delay licensing until the ANN or compact-index phase exists.

## Chosen option

Option 2: publish under the MIT License.

## Pros

- Learners can reuse the baseline, docs, and verification notes with a standard
  permissive license.
- The public repository now has a legal contract that matches its didactic goal.

## Cons

- Downstream forks may copy the exact baseline without preserving the same
  benchmark caveats.

## Consequences

- The repo remains an honest correctness-first baseline.
- Future optimization work can build on a reusable public reference.
