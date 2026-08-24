# pallet-content Documentation

This folder contains detailed documentation for operators, builders, and indexer
integrators.

- [`api-reference.md`](./api-reference.md)
  - Complete runtime interface surface.
- [`indexing-workflow.md`](./indexing-workflow.md)
  - Event-driven projection and integration flow for
    [`acuity-index`](https://github.com/acuity-network/acuity-index).
- [`runtime-integration.md`](./runtime-integration.md)
  - Practical steps for runtime configuration.
- [`lifecycle.md`](./lifecycle.md)
  - Stateful view of allowed flag transitions and errors.

## Dependency Pinning

This project uses `polkadot-sdk` from git tag `polkadot-stable2606-1`.

- Keep FRAME/Substrate dependencies on the same git source/tag across runtime
  crates.
- Avoid mixing crates.io and git-sourced `polkadot-sdk` dependencies in one
  workspace.

The main project documentation remains in [`../README.md`](../README.md).
