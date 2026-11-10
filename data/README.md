# Generated tables

`minecraft-v1.json` is a dataset in the [table format](../spec/table-format.md)
(`schema_version` 1, `data_version` 1) for the ten [target configurations](../spec/configurations.md):
Minecraft 1.21 and 1.21.11 × standard Vanilla/Fabric/Forge, Lunar and Lunar + Fabric.

It is produced by `mc_mappings_generator` (repository `mc_mappings_generator`,
`consumers/mc_mappings/build-dataset.sh OUT 1`) from the draft Minecraft v1 catalog
(`consumers/mc_mappings/catalog-v1.json` there), pending the frozen list of mc-bridge ticket B1.
Every catalog symbol is resolved and verified in every configuration; identical tables are shared
(6 profiles for 10 configurations).

`minecraft-v1.provenance.json` records, per configuration, the generated file's SHA-256 and its
upstream inputs (URLs, SHA-256, licences, loader builds, Lunar class-cache hashes) and the proof of
the runtime namespace. Observed runtime namespaces: standard Vanilla `official` (obfuscated),
standard and Lunar Fabric `intermediary`, standard Forge and Lunar without loader `mojang`.

Runtime names come from Mojang mappings (no redistribution by the generator) and Intermediary
(CC0): check the licence column of the provenance before publishing this file.
