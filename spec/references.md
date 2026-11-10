# Sources and evidence

These sources concern the [target configurations](configurations.md) for **1.21 and 1.21.11**. They establish the availability of distributions or modules; they do not, on their own, validate runtime names and signatures.

## Standard launcher

- **Mojang**: [official manifest](https://piston-meta.mojang.com/mc/game/version_manifest_v2.json) and [target version descriptors](versions.md). The descriptors' SHA-1 hashes were checked against the manifest. Download details for the client and client_mappings are retained in the [collected data](evidence/data.json), under entries 1.21 and 1.21.11.
- **Fabric**: official endpoints for [1.21](https://meta.fabricmc.net/v2/versions/loader/1.21) and [1.21.11](https://meta.fabricmc.net/v2/versions/loader/1.21.11). The selected references have loader.stable and intermediary.stable set to true, with an Intermediary version matching Minecraft exactly.
- **Forge**: [official Maven index](https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml), corroborated by the download pages for [1.21](https://files.minecraftforge.net/net/minecraftforge/forge/index_1.21.html) and [1.21.11](https://files.minecraftforge.net/net/minecraftforge/forge/index_1.21.11.html).

| Minecraft | Fabric Loader | Intermediary | Forge |
| --- | --- | --- | --- |
| 1.21 | 0.20.1 | 1.21 | 1.21-51.0.33 |
| 1.21.11 | 0.20.1 | 1.21.11 | 1.21.11-61.2.5 |

These builds are recorded release references, not recommended versions or a guarantee of compatibility with every loader build. This collection does not establish that the game was launched or mappings were tested. The JSON extracts and response hashes identify the references consulted; they are not a complete archive of the responses.

## Lunar

The [user's launcher report](evidence/lunar-user-report.json) confirms that both target versions are offered with the **Lunar** and **Lunar + Fabric** options, with no Forge module for these versions.

This evidence comes from the user's launcher check, without a screenshot or an independent check by the agent. **The exact launcher, Lunar client, and Fabric loader builds were not provided.** They are neither invented nor inferred from a changelog. The report confirms module availability, not runtime mappings or signatures.

For table preparation, the [Lunar Mapping Project](https://github.com/Lunar-Mapping-Project/mappings) is a candidate source of mappings for Lunar's internal classes. The [examined revision](https://github.com/Lunar-Mapping-Project/mappings/tree/4b1258c5fbe63fb818b364302e3d252ddbf42f5b) and consulted releases are recorded in the [community evidence](evidence/lunar-community.json). They target specific Lunar files and do not demonstrate complete coverage of the target configurations. No mappings are incorporated here; any reuse must preserve provenance and applicable license notices.

## Builds used for the v1 tables

The bundled tables (`data_version` 1, 2026-11-10) were generated from these builds; their hashes
and the runtime-namespace proofs are in `data/minecraft-v1.provenance.json`.

| Configuration | Builds |
| --- | --- |
| Standard Vanilla | Mojang client jars and `client_mappings` of 1.21 and 1.21.11 |
| Standard Fabric | Fabric Loader 0.20.1, Intermediary 1.21 / 1.21.11 |
| Standard Forge | 1.21-51.0.33 (latest; 1.21 has no recommended build), 1.21.11-61.2.4 (recommended; 61.2.5 above declares the same `install_profile` processors) |
| Lunar, Lunar + Fabric | Lunar launcher 3.7.34; `genesis-0.1.0-SNAPSHOT-all.jar` checksum `a425c132`, `lunar.jar` `bc92bf50`, `lunar-platform-mappings-v1_21_0.jar` `bcd49680`, `lunar-platform-mappings-v1_21_11.jar` `302edf01`; Fabric module loader 0.20.1 |

Lunar runtime names were detected from Lunar's own class cache after launching each
configuration (scores 1.0 for classes and members): Mojang names without a loader, Intermediary
names with Fabric. The Lunar module without a loader therefore differs from standard Vanilla.
A Lunar update that changes these names is fixed by regenerating the tables and publishing a new
`data_version`, as described in the [scope](README.md#resolution-and-data).
