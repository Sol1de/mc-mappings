# mc-mappings specifications

## Documents

- [Target versions](versions.md).
- [Target configurations](configurations.md).
- [Sources, evidence, and limitations](references.md).
- [Resolution model and identifiers](model.md).
- [Table format](table-format.md).

Implementation and validation progress is tracked in tickets and issues. These documents define the intended coverage without duplicating progress tracking.

The Rust model and structural/local JSON validation are implemented in the crate.
Cross-table auditing, resolution, and production mapping tables are not provided yet.

## Scope

- Minecraft Java: **1.21 and 1.21.11**.
- Standard launcher: Vanilla, Fabric, and Forge.
- Lunar: **Lunar** and **Lunar + Fabric** modules. According to the user's launcher report, no Forge module is offered for these versions.
- Windows x64 only.
- Snapshots, pre-releases, release candidates, and NeoForge are excluded.

The standard launcher refers to a standard installation without Lunar-specific transformations. The Lunar module without Fabric is not assumed to be identical to standard Vanilla.

## Resolution and data

- Resolve class, field, and method names and their signatures using versioned tables, without a JVM, JNI, or JVMTI.
- Do not depend on any other crate in the catalog.
- Select tables by exact Minecraft version, launcher, and loader, without silently falling back to a nearby version.
- Do not include automatic discovery, diagnostic tools, or a plugin system in the crate.
- Keep source documentation and available build references separately from the mapping tables; builds are not additional selection keys at this stage.
- If a Lunar update causes an incompatibility, correct the tables through an issue and a crate release. Distinguish between builds only when a concrete need requires it.

A configuration's availability does not establish the validity of its mappings. Runtime names and signatures must be checked for the required symbols before support is announced.

## Coverage changes and regression checks

1. Add a version or profile only through an explicit decision, based on actual needs. A new Minecraft release does not automatically extend coverage.
2. Update the target versions, configuration list (which replaces the matrix), and supporting evidence. Check the distributions and modules actually offered; do not assume that every combination exists.
3. Prepare the new configuration's tables and tests. Document sources and redistribution conditions separately from the table format.
4. **Retain all previously supported versions and configurations**, along with their tables and tests. A new version never replaces the previous one. Changes to a launcher's distributions alone do not justify removing existing tables.
5. Run regression tests for all previously supported configurations, as well as tests for the new tables, before publishing. Every fix must also preserve this coverage.

Update the relevant documents, data, and tests together; tickets and issues track their progress.

## Preparing for older versions without implementing them

The common table format and semantic identifiers remain independent of any particular provider's names and formats. Source conversion takes place during data preparation, outside resolution.

This allows Mojang, Fabric, MCP/SRG, or community mappings to be added later as needed. Source documentation and redistribution conditions are handled outside the table format. No older-version support is added now.

Mappings translate names and signatures; behavioral or API differences between versions are handled by the consuming layers.
