# Resolution model and identifiers

This document defines the resolution model and its data contract. The [table format](table-format.md) specifies its serialized representation.

## Configuration selection

A configuration has three independent identity components:

| Component | Representation | Meaning |
| --- | --- | --- |
| Minecraft version | Exact string | A version explicitly listed in the registry |
| Launcher | `standard` or `lunar` | Standard installation or Lunar transformations |
| Loader | `none`, `fabric`, or `forge` | The loader used by that configuration |

`lunar` with loader `none` represents the Lunar module without Fabric. It does not identify the standard Vanilla binary. The [target configurations](configurations.md) define the intended coverage; their availability does not mean that validated tables already exist.

Selection follows these rules:

1. Normalize the exact input `1.21.0` to `1.21`. This is the only alias. Do not trim whitespace, change case, remove leading zeroes, or apply general semantic-version normalization.
2. Look up the exact `(minecraft_version, launcher, loader)` tuple in the mapping registry.
3. Use only the profile referenced by that registry entry. No match means an unsupported configuration. Multiple matches mean invalid data, never an arbitrary choice.

All unlisted version strings receive the same unsupported-configuration result. There is no distinction between an unlisted stable version, a snapshot, a pre-release, a release candidate, or another unrecognized string. Non-string input is a format error. A recognized version with an unlisted launcher/loader combination is also unsupported.

Stored versions are canonical: registry entries use `1.21`, never its alias. Versions outside the explicitly approved [scope](versions.md) cannot be published merely by inserting registry entries.

## Naming profiles and builds

A `profile_id` identifies one exact table of runtime names and signatures within a data release. It is an opaque identifier, not a provider namespace or an instruction to infer runtime names. Its contents are interpreted within that release's `data_version`.

Different configurations may reference the same profile only after their runtime names and signatures have been verified identical. Their configuration identities remain separate. A launcher or loader alone never determines a naming namespace.

Available build references may be documented separately from the mapping tables; they are not part of the selection tuple. There is no build selector, wildcard, precedence rule, or implicit default in this format.

If incompatible builds later require distinct tables, the format and selection contract must be revised explicitly to distinguish them. Separate profiles can hold different tables, but the current registry must not assign multiple profiles to the same configuration. Until that change is specified, fix incompatible tables through an issue and a crate release, as described in the [scope](README.md#resolution-and-data).

## Semantic identities

`ClassId`, `FieldId`, and `MethodId` are distinct kinds of semantic identifier. They identify a concept used by consumers, independently of Minecraft version and provider names. A runtime name must never serve as the implicit identity of a symbol.

Identifiers use the case-sensitive grammar `[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*`. Empty segments, whitespace, uppercase characters, and alternate spellings are invalid. Do not normalize identifiers or guess aliases.

Each identifier occurs once across the entire semantic catalog, including across kinds. Its kind is fixed. Never reuse a published identifier for a different meaning. A field or method identifier may contain its conceptual owner's name for readability, but the runtime declaring class is an explicit table reference, not inferred from that spelling.

Each method overload receives its own semantic identifier. A lookup requires the identifier of the intended overload; there is no lookup by a bare runtime method name. Identifiers remain stable when a provider renames a symbol. Changes in behavior or API semantics are handled by consuming layers rather than hidden in a mapping.

The catalog declares the symbols required in this dataset. Every profile must cover it completely. Runtime declaring classes, names, static/instance status, and signatures are recorded per profile because they may differ across configurations. Shared tables are allowed; partial support is not silently substituted for required symbols.

## Types and signatures

Field types and method signatures are structured data:

- A primitive is one of `boolean`, `byte`, `char`, `short`, `int`, `long`, `float`, or `double`.
- A mapped class is a reference to a `ClassId` in the same profile.
- An external class has an explicit runtime internal name and requires no mapping, for example `java/lang/String`. Do not use an external class entry to bypass a mapped class reference.
- An array contains another non-void type; nested arrays are represented recursively.
- `void` is permitted only as a method return type.

A method has ordered parameter types and one return type. A field has one non-void type. Each member also records its runtime declaring class and whether it is static. Runtime class names use slash-separated internal names, not dotted names or object descriptors. These strings are data, not JNI handles.

The resolver translates mapped class references throughout signatures, including inside arrays, to produce complete runtime descriptors. Provider formats are converted during data preparation, outside resolution.

## Versions

| Version | Responsibility |
| --- | --- |
| Crate version | The library release and its public API; recorded by the package, not duplicated in each table |
| `schema_version` | The serialized structure and interpretation rules; initially integer `1` |
| `data_version` | A positive, monotonically increasing integer identifying a published dataset revision, independently of the crate version |

A published `data_version` is immutable. Any change to the catalog, registry, or profiles requires a new value. A format change also requires a new `schema_version`. A crate may be released without changing its data; changing data requires publishing a crate release containing that new dataset. Minecraft versions are separate from all three values.

Mapping tables contain only the data needed for resolution. Source documentation, available build references, and redistribution conditions are maintained separately from this format, without requiring a metadata record for each profile. Existing availability evidence is not proof of runtime mapping validity. Source conversion, integrity verification, and reproducible generation belong to data preparation.

## Consumer catalog

The list of symbols needed by mc-bridge must be derived from its public operations. mc-mappings does not need that complete list to define this model, and it must not depend on the mc-bridge crate to consume it.

The example in the table format uses synthetic symbols only. It is not the Minecraft v1 catalog and does not claim support for a real configuration. Real tables must define the agreed symbols before their support can be announced.

## Validation cases for the eventual implementation

These cases document expected behavior; no executable tests or Rust implementation are introduced by this specification.

| Case | Expected result |
| --- | --- |
| Registered `1.21` configuration | Select its exact profile |
| `1.21.0` for the same configuration | Select the same profile after normalization |
| Registered `1.21.11` configuration | Select its exact profile, independently of `1.21` |
| Unlisted `1.21.1`, `1.21-pre1`, `1.21-rc1`, `24w14a`, `1.21.x`, `01.21`, or ` 1.21` | Unsupported configuration, with no fallback or version categorization |
| Missing launcher/loader combination | Unsupported configuration |
| Invalid semantic identifier | Invalid identifier |
| Duplicate catalog ID or configuration tuple | Invalid data |
| Missing profile, symbol, declaring class, or mapped type reference | Invalid data |
| Two methods with the same owner, runtime name, and complete runtime descriptor | Invalid data: ambiguous runtime mapping |
| Unsupported schema or malformed table structure | Invalid data; never interpret as a different schema |

The model contains no JavaVM, JNIEnv, JNI/JVMTI calls, runtime classloader objects, or dependency on another catalog crate. Names and signatures remain plain data.
