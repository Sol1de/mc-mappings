# Table format

This document defines JSON schema version `2` for the [resolution model](model.md). It is a documented format, not an implementation or a set of production mappings.

Runtime names and descriptor constraints refer to the [Java 21 JVM specification, sections 4.2 and 4.3](https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.2). The JSON representation below is specific to mc-mappings.

## Dataset structure

One UTF-8 JSON document contains the catalog, registry, and profiles. Their separation is logical; no file loader, include mechanism, or generated binary representation is specified here.

All fields shown below are required, except `catalog.scoped`. Objects reject unknown fields and duplicate keys. Arrays may be empty unless a rule below requires entries. Values must have the declared JSON type; do not coerce strings, numbers, or booleans.

| Root field | Type | Meaning |
| --- | --- | --- |
| `schema_version` | Integer | `2`. Schema `1` is still read: it is the same format without `catalog.scoped` |
| `data_version` | Positive integer | Immutable dataset release identifier |
| `catalog` | Object | Arrays `classes`, `fields`, and `methods`, each containing semantic ID strings, and the optional array `scoped` |
| `configurations` | Array of objects | Exact configuration-to-profile registry |
| `profiles` | Array of objects | Runtime tables |

A configuration has four string fields: `minecraft_version`, `launcher`, `loader`, and `profile_id`. Launcher and loader values are defined in the model. The first three fields form its unique key. Stored versions must use the canonical spelling.

A `catalog.scoped` entry has `id` (a catalog ID string of any kind) and `minecraft_versions` (a nonempty array of canonical version strings). It states that the symbol exists only in those Minecraft versions; a symbol without an entry exists in every version. An ID has at most one entry.

A profile has `profile_id`, `classes`, `fields`, and `methods`. Profile IDs use the semantic identifier grammar and must be unique within the dataset, but form a separate namespace from catalog IDs.

## Symbol records

| Array | Record fields |
| --- | --- |
| `classes` | `id` (ClassId string), `runtime_name` (internal class name string) |
| `fields` | `id` (FieldId string), `owner` (ClassId string), `runtime_name` (member name string), `static` (boolean), `type` (type object) |
| `methods` | `id` (MethodId string), `owner` (ClassId string), `runtime_name` (member name string), `static` (boolean), `parameters` (array of type objects), `returns` (type object) |

Each profile contains exactly one record for each catalog ID of the corresponding kind that exists in the Minecraft versions of the configurations using the profile, with no additional IDs. Configurations of versions that do not have the same symbols cannot share a profile. Every owner and mapped type reference resolves to a class in that profile. Owners identify the runtime declaring class; inheritance discovery is not part of the format.

Runtime names are nonempty and contain no whitespace. Class names contain no `.`, `;`, or `[` and use nonempty slash-separated segments. Member names contain none of `.`, `;`, `[`, or `/`. Method names may use the special names `<init>` and `<clinit>`; otherwise angle brackets are forbidden. Field names cannot contain angle brackets. Constructors are non-static and return void; class initializers are static, have no parameters, and return void. This representation does not promise that consumers can invoke a class initializer directly.

Within a profile, class runtime names must be unique. Field mappings must be unique by runtime owner, name, and resolved type descriptor. Method mappings must be unique by runtime owner, name, and complete resolved descriptor. Static/instance status does not distinguish otherwise identical mappings. A collision is invalid data, not an alias.

## Type objects

Each type object has exactly the fields in its row. There are no implicit class references or raw descriptor strings.

| Type | JSON representation |
| --- | --- |
| Primitive | `{"kind":"primitive","name":"int"}` with a primitive name from the model |
| Mapped class | `{"kind":"class","id":"example.client"}` |
| External class | `{"kind":"external","runtime_name":"java/lang/String"}` |
| Array | `{"kind":"array","element":{"kind":"primitive","name":"int"}}` |
| Void return | `{"kind":"void"}` |

External class names follow the class-name rules above. If a runtime class is already mapped in the profile, refer to its ClassId instead. `void` is invalid for a field, parameter, or array element. Nested arrays must not exceed 255 dimensions. Class references in parameter and return types are resolved in the selected profile, never a different configuration's table.

## Synthetic example

All runtime names and symbols below are fictional. The configuration illustrates registry syntax only; this document provides no validated Minecraft mappings. The example uses `data_version: 1` to illustrate the format, not to reserve a real data release.

```json
{
  "schema_version": 2,
  "data_version": 1,
  "catalog": {
    "classes": ["example.client"],
    "fields": ["example.client.counter"],
    "methods": ["example.client.find", "example.client.find_by_index"]
  },
  "configurations": [
    {
      "minecraft_version": "1.21",
      "launcher": "standard",
      "loader": "none",
      "profile_id": "example.standard"
    }
  ],
  "profiles": [
    {
      "profile_id": "example.standard",
      "classes": [
        {"id": "example.client", "runtime_name": "example/runtime/Client"}
      ],
      "fields": [
        {
          "id": "example.client.counter",
          "owner": "example.client",
          "runtime_name": "a",
          "static": false,
          "type": {"kind": "primitive", "name": "int"}
        }
      ],
      "methods": [
        {
          "id": "example.client.find",
          "owner": "example.client",
          "runtime_name": "b",
          "static": false,
          "parameters": [
            {"kind": "external", "runtime_name": "java/lang/String"}
          ],
          "returns": {"kind": "class", "id": "example.client"}
        },
        {
          "id": "example.client.find_by_index",
          "owner": "example.client",
          "runtime_name": "b",
          "static": false,
          "parameters": [{"kind": "primitive", "name": "int"}],
          "returns": {
            "kind": "array",
            "element": {"kind": "class", "id": "example.client"}
          }
        }
      ]
    }
  ]
}
```

Changing both methods to the same owner, runtime name, parameters, and return type makes this dataset invalid even though their semantic IDs differ. Removing `example.client` from the profile also makes it invalid because the catalog, owners, and return types reference it.

To state that `example.client.find_by_index` exists in 1.21 only, the catalog gains `"scoped": [{"id": "example.client.find_by_index", "minecraft_versions": ["1.21"]}]`. A profile used by a 1.21.11 configuration then has no record for it, and resolving it there reports that the symbol is absent in that version.

The example deliberately omits the other target configurations. Production data must cover all configurations and symbols required for the support it announces; an illustrative registry is not evidence of coverage.
