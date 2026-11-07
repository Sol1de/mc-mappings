# Target Minecraft versions

The intended coverage includes **1.21 and 1.21.11**, for the [listed configurations](configurations.md).

| Version | Java version in the official profile | Mojang version descriptor |
| --- | --- | --- |
| 1.21 | 21 | [Metadata](https://piston-meta.mojang.com/v1/packages/f7840475dae7c9ca4968b8198055db8d708a162d/1.21.json) |
| 1.21.11 | 21 | [Metadata](https://piston-meta.mojang.com/v1/packages/35f8a9bc4874e2c95e51b3ad21e043a33776f7c2/1.21.11.json) |

The descriptors come from the [official Mojang manifest](https://piston-meta.mojang.com/mc/game/version_manifest_v2.json). Each contains a client and a client_mappings reference; their download details and hashes are retained in the [evidence](references.md).

Versions are exact identities: 1.21 does not refer to the entire 1.21.x series. The existence of Mojang mappings does not guarantee that Fabric, Forge, or Lunar use the same names at runtime. Rules for adding versions and preserving support are in the [README](README.md#coverage-changes-and-regression-checks).
