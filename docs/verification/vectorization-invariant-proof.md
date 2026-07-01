# Vectorization Invariant Proof

The repository already proved the official worked examples. That was necessary,
but it was not a strong enough guardrail for refactoring a dense 14-dimension
transformation.

This verification layer adds property-style contract evidence for the
vectorizer:

- when `last_transaction` is missing, indices `5` and `6` stay at the required
  `-1` sentinel
- when `last_transaction` is present, every dimension stays within the expected
  normalized range
- unrecognized MCCs still fall back to the documented default risk `0.5`
- clamped fields remain bounded even when the raw request values are far above
  the normalization constants

## Why keep this proof

Pros:

- catches silent contract drift that example-only tests can miss
- makes the repo teach the normalization rules as invariants, not just as
  snapshots
- gives future optimization work a safer baseline before ANN or memory-layout
  changes

Cons:

- property-style tests add one more dependency and slightly longer test time
- they do not replace full-dataset accuracy or performance evidence

## Run

```bash
cargo test --test vector_contract
```
