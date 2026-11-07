# mc-mappings

Rust data model and strict JSON parser for the [mapping table format](spec/table-format.md).
No JVM, JNI or dependency on another catalog crate is required.

```rust
use mc_mappings::{ClassId, Dataset, normalize_minecraft_version};

let id: ClassId = "example.client".parse()?;
assert_eq!(id.as_str(), "example.client");
assert_eq!(normalize_minecraft_version("1.21.0"), "1.21");
let tables = Dataset::from_json(include_str!("tests/fixtures/synthetic.json"))?;
assert_eq!(tables.data_version.get(), 1);
```

`Dataset::from_json` rejects malformed JSON, unknown or duplicate fields, missing
fields, incorrect JSON types, invalid identifiers, unsupported schemas and invalid
local names or signatures. Arrays support up to 255 dimensions. Use this entry
point for the full supported nesting depth; generic `serde_json::from_str` keeps
serde_json's lower default recursion limit. The public records can be constructed
or edited in Rust; call `Dataset::validate` afterward to recheck local constraints.

Stored configurations must use canonical versions `1.21` or `1.21.11` and the
combinations in [spec/configurations.md](spec/configurations.md). The normalization
helper applies only to selection inputs; it neither selects a profile nor makes an
unregistered configuration supported. The crate version lives in `Cargo.toml`,
`schema_version` is currently `1`, and `data_version` is an independent positive
`u64` dataset revision. This parser cannot verify publication history or immutability.

MM-2 implements structural and local validation only. Cross-table references,
duplicate semantic IDs, configuration/profile uniqueness, runtime collisions and
catalog completeness still require the MM-7 audit. Successful parsing does not
establish that a dataset is consistent or ready for publication. No real mappings,
resolver, generation pipeline or CI are included yet. The fixture is synthetic.

Run `cargo test` and `cargo clippy --all-targets -- -D warnings` to check the crate.
