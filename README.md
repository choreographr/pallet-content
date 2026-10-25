# pallet-content

`pallet-content` is a Substrate FRAME pallet that publishes and versions
content-item references on-chain. It manages ownership, revision numbering, and
item lifecycle flags while emitting rich events that can be consumed by indexers.

It is intended to be used together with
`acuity-index` (`https://github.com/acuity-network/acuity-index`) for
off-chain indexing and query-serving of the emitted content events.

## Workspace pallets

This workspace also includes companion pallets that build on the core content
registry:

- `pallet-account-content`
  - account-scoped ordered lists of owned content items
  - see `pallet-account-content/README.md`
- `pallet-account-profile`
  - one profile pointer per account into owned content
  - see `pallet-account-profile/README.md`
- `pallet-content-reactions`
  - account-scoped emoji reactions on item revisions
  - see `pallet-content-reactions/README.md`

## Design overview

The pallet intentionally keeps storage compact and stores only immutable control
state for each item:

- The current item owner
- The latest revision number
- Current lifecycle flags (revisionable/retractable/retracted)

Actual content payloads (for example, IPFS CIDs) and graph relations are passed
through event fields and can be materialized externally by an indexer.

### Why this shape

- Low on-chain state size: only what is needed for governance and state checks
  is persisted.
- Deterministic item identity: an item ID is derived from account + nonce so clients
  can compute expected IDs and subscribe to them before content is stored.
- Indexing-friendly events: off-chain systems can reconstruct revision history,
  link relationships, and parent relationships from emitted events.

## Types

- `ItemId([u8; 32])`
  - Deterministic identifier for an item.
  - Calculated as `blake2_256(account.encode() || nonce.encode() || item_id_namespace.encode())`.

- `IpfsHash([u8; 32])`
  - Opaque fixed 32-byte payload reference that indexes content storage
    (typically IPFS-style digest bytes).

- `Nonce([u8; 32])`
  - Caller-supplied 32-byte nonce used in `publish_item` to derive `ItemId`.

- `Item<AccountId>`
  - `owner`: item owner account.
  - `revision_id`: current latest revision index.
  - `flags`: state flags bitfield.

## Storage

- `ItemState: StorageMap<ItemId, Item<AccountId>>`
  - Single source of truth for ownership, revision counter, and flags.

## Item flags

The `flags` field is a bitmask:

- `REVISIONABLE = 1 << 0`
  - New revisions are allowed.
- `RETRACTABLE = 1 << 1`
  - Item can be marked retracted.
- `RETRACTED = 1 << 2`
  - Set by `retract_item`.

`publish_item` only accepts flags within `REVISIONABLE | RETRACTABLE`; reserved
bits are rejected.

## Extrinsics

All calls are signed and weight-charged via their corresponding `WeightInfo`
methods.

- `publish_item(origin, nonce, parents, flags, links, mentions, ipfs_hash)`
  - Creates a new item.
  - Validates `flags`.
  - Fails if the derived `ItemId` already exists.
  - Stores the item and emits:
    - `PublishItem { item_id, owner, parents, flags }`
    - `PublishRevision { item_id, owner, revision_id: 0, links, mentions, ipfs_hash }`

- `publish_revision(origin, item_id, links, mentions, ipfs_hash)`
  - Validates existence and ownership.
  - Fails if item is retracted or not revisionable.
  - Increments `revision_id` and emits:
    - `PublishRevision { item_id, owner, revision_id, links, mentions, ipfs_hash }`

- `retract_item(origin, item_id)`
  - Marks item as retracted.
  - Allowed only by owner, only once, and only when `RETRACTABLE` is enabled.
  - Emits `RetractItem { item_id, owner }`.

- `set_not_revisionable(origin, item_id)`
  - Clears the `REVISIONABLE` bit.
  - Allowed only by owner and only if already revisionable.
  - Emits `SetNotRevsionable { item_id, owner }`.

- `set_not_retractable(origin, item_id)`
  - Clears the `RETRACTABLE` bit.
  - Allowed only by owner and only if currently retractable.
  - Emits `SetNotRetractable { item_id, owner }`.

## Errors

- `ItemAlreadyExists`
- `ItemNotFound`
- `ItemRetracted`
- `ItemNotRevisionable`
- `ItemNotRetractable`
- `WrongAccount`
- `InvalidFlags`
- `RevisionIdOverflow`

## Events

- `PublishItem`
  - New top-level item creation event.
- `PublishRevision`
  - Emitted for initial publish and every revision update.
- `RetractItem`
  - Lifecycle transition into retracted state.
- `SetNotRevsionable`
  - Revision permission removed.
- `SetNotRetractable`
  - Retraction permission removed.

## Integration with acuity-index

`pallet-content` is intentionally designed as a clean event producer for
indexing. The companion repository [`acuity-index`](https://github.com/acuity-network/acuity-index)
is where event streams can be consumed and transformed into queryable state.

Recommended index behavior:

- Subscribe to all `pallet_content` events.
- On `PublishItem`, create a canonical item record from the event payload.
- On `PublishRevision`, append revision history and update latest revision.
- On state-change events (`RetractItem`, `SetNotRevsionable`, `SetNotRetractable`),
  update indexing permissions and lifecycle metadata.

`parents`, `links`, `mentions`, and `ipfs_hash` are not persisted in pallet storage, so they
must be indexed from events.

## Runtime integration

Add the pallet and configure weight info in your runtime:

```rust
impl pallet_content::Config for Runtime {
    type WeightInfo = pallet_content::SubstrateWeight<Runtime>;
    type ItemIdNamespace = frame_support::traits::ConstU32<0>;
    type MaxParents = frame_support::traits::ConstU32<64>;
    type MaxLinks = frame_support::traits::ConstU32<256>;
    type MaxMentions = frame_support::traits::ConstU32<256>;
}
```

Then add the pallet to the runtime `construct_runtime!` and include it in your
runtime crate dependencies as usual.

This repository pins FRAME/Substrate dependencies to `polkadot-sdk` 2609.0.0
from crates.io (upstream git tag `polkadot-stable2609`). Keep your runtime on the
same version so Cargo does not resolve mixed registry/git variants.

### Additional documentation

For a deeper reference and indexer guidance, see:

- [API Reference](docs/api-reference.md)
- [Indexing Workflow](docs/indexing-workflow.md)
- [Runtime Integration Guide](docs/runtime-integration.md)
- [Item Lifecycle](docs/lifecycle.md)

## Security and validation

- Item ID derivation is deterministic and deterministic replay-safe with the same
  account + nonce pair.
- Flags are strictly bit-checked at creation.
- Overflow is checked when incrementing `revision_id`.

### Security auditing

This repository uses `cargo-audit` with a project-local policy file at
`.cargo/audit.toml`.

- Known transitive advisories from pinned upstream dependencies are tracked via
  explicit `ignore` entries.
- The policy is intentionally strict (`deny = ["warnings"]`) so newly introduced
  advisories fail audit checks unless explicitly reviewed.
- Yanked crate checks are currently disabled because the affected crates are
  transitive from the pinned SDK set.

### Run audit locally

```bash
cargo audit
```

### Policy maintenance

When updating `polkadot-sdk` dependencies, re-run `cargo audit` and remove any
ignore entries that are no longer needed.
