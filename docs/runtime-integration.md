# Runtime Integration Guide

Use this document to wire `pallet-content` into a Substrate runtime.

## 1. Add dependencies

In your runtime `Cargo.toml`:

```toml
[dependencies]
pallet-content = { version = "0.1.0", default-features = false }
polkadot-sdk = { version = "2609.0.0", default-features = false }
```

And enable relevant runtime features (`std` for host tests/dev, otherwise keep
`default-features = false`).

Use the same `polkadot-sdk` version across your runtime crates to avoid mixing
registry and git-sourced FRAME dependencies. Version 2609.0.0 is published from
upstream git tag `polkadot-stable2609`; prefer the crates.io crate, and use the
tag only if you already build FRAME from git.

## 2. Add pallet to `construct_runtime!`

```rust
construct_runtime!(
    pub enum Runtime {
        // ...
        Content: pallet_content,
    }
);
```

## 3. Configure `Config`

```rust
impl pallet_content::Config for Runtime {
    type WeightInfo = pallet_content::SubstrateWeight<Runtime>;
    type ItemIdNamespace = frame_support::traits::ConstU32<0>;
    type MaxParents = frame_support::traits::ConstU32<64>;
    type MaxLinks = frame_support::traits::ConstU32<256>;
    type MaxMentions = frame_support::traits::ConstU32<256>;
}
```

## 4. Runtime feature wiring

If your runtime conditionally compiles benchmarks:

```rust
#[cfg(feature = "runtime-benchmarks")]
type ContentBenchmarking = pallet_content::benchmarking::Pallet<Runtime>;
```

## 5. Exposed types to indexers

Expose these runtime types as needed:

- `ItemId`
- `IpfsHash`
- `Nonce`
- `Item`

Then import and project pallet events in RPC/indexing layers as required.

## 6. Migrate carefully

- If you change flag semantics, preserve backward-compatible behavior for old
  revision history in external indexers.
- Document any external expectations about `parents`, `links`, `mentions`, and `ipfs_hash`
  because they are emitted only via events, not storage.
