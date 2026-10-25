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

FRAME/Substrate dependencies are pinned to a single `polkadot-sdk` umbrella
version, declared once in `[workspace.dependencies]` of the root `Cargo.toml`:

```toml
polkadot-sdk = { version = "2609.0.0", default-features = false }
```

The crate is sourced from **crates.io**, not a git tag, so every crate in the
workspace resolves one SDK version and no SDK crate is version-pinned
per-manifest. Version 2609.0.0 is published from upstream git tag
`polkadot-stable2609`; a `-N` suffix marks a patch release within the same train,
so crate `2606.1.0` corresponds to tag `polkadot-stable2606-1`.

- Keep FRAME/Substrate dependencies on the same `polkadot-sdk` version across
  runtime crates.
- Avoid mixing crates.io and git-sourced `polkadot-sdk` dependencies in one
  workspace.

The main project documentation remains in [`../README.md`](../README.md).
