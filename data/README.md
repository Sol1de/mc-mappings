# Generated tables

`minecraft-v1.json` is a dataset in the [table format](../spec/table-format.md)
(`schema_version` 2, `data_version` 2) for the ten [target configurations](../spec/configurations.md):
Minecraft 1.21 and 1.21.11 × standard Vanilla/Fabric/Forge, Lunar and Lunar + Fabric.

It is produced by `mc_mappings_generator` (repository `mc_mappings_generator`,
`consumers/mc_mappings/build-dataset.sh OUT 2`) from the Minecraft v1 catalog
(`consumers/mc_mappings/catalog-v1.json` there), listed in [spec/catalog.md](../spec/catalog.md).
Load it with `Dataset::minecraft_v1()`; check it with `cargo run --example audit`.
Every catalog symbol is resolved and verified in every configuration of the Minecraft versions it
exists in; identical tables are shared (6 profiles for 10 configurations: per version, one for
standard Vanilla, one for both Fabric configurations, one for Forge and Lunar without loader).

`minecraft-v1.provenance.json` records, per configuration, the generated file's SHA-256 and its
upstream inputs (URLs, SHA-256, licences, loader builds, Lunar class-cache hashes) and the proof of
the runtime namespace. Observed runtime namespaces: standard Vanilla `official` (obfuscated),
standard and Lunar Fabric `intermediary`, standard Forge and Lunar without loader `mojang`.

Runtime names come from Mojang mappings, Intermediary (CC0) and the Lunar class cache; they are
distributed as-is under the notice in the [README](../README.md#licence).
