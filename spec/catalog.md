# Minecraft v1 symbol catalog

`data_version` 1 of the [bundled tables](../data/README.md) declares the symbols below: the
Minecraft API v1 needed by mc-bridge (session, player, world, entities, inventory and the game
thread). Every [target configuration](configurations.md) resolves all of them; no configuration
has partial support.

The list is the data contract proposed to mc-bridge (ticket B1). It is maintained as the
`mc_mappings_generator` input `consumers/mc_mappings/catalog-v1.json`, which names each symbol by
its Mojang name and its semantic types. Adding, removing or renaming a symbol means editing that
file, regenerating the tables and publishing a new `data_version`; published identifiers are never
reused for another meaning.

| Family | Identifiers |
| --- | --- |
| Session and game thread | `minecraft.client` (class), `minecraft.blockable_event_loop` (class), `minecraft.user` (class), `minecraft.client.player` (field), `minecraft.client.level` (field), `minecraft.client.game_mode` (field), `minecraft.client.get_instance`, `minecraft.client.get_user`, `minecraft.client.tick`, `minecraft.blockable_event_loop.execute`, `minecraft.blockable_event_loop.is_same_thread`, `minecraft.user.get_name`, `minecraft.user.get_profile_id` |
| Player | `minecraft.player` (class), `minecraft.local_player` (class), `minecraft.player.container_menu` (field), `minecraft.player.get_inventory` |
| Entities | `minecraft.entity` (class), `minecraft.entity_type` (class), `minecraft.living_entity` (class), `minecraft.entity.get_id`, `minecraft.entity.get_uuid`, `minecraft.entity.get_type`, `minecraft.entity.get_x`, `minecraft.entity.get_y`, `minecraft.entity.get_z`, `minecraft.entity.position`, `minecraft.entity.get_y_rot`, `minecraft.entity.get_x_rot`, `minecraft.entity.set_y_rot`, `minecraft.entity.set_x_rot`, `minecraft.entity.get_delta_movement`, `minecraft.entity.set_delta_movement`, `minecraft.entity.is_alive`, `minecraft.entity.is_removed`, `minecraft.entity.on_ground`, `minecraft.entity_type.get_description_id`, `minecraft.living_entity.get_health`, `minecraft.living_entity.get_max_health` |
| World | `minecraft.level` (class), `minecraft.client_level` (class), `minecraft.resource_key` (class), `minecraft.resource_location` (class), `minecraft.vec3` (class), `minecraft.vec3.x` (field), `minecraft.vec3.y` (field), `minecraft.vec3.z` (field), `minecraft.client_level.entities_for_rendering`, `minecraft.level.dimension`, `minecraft.resource_key.location` |
| Inventory | `minecraft.inventory` (class), `minecraft.item_stack` (class), `minecraft.item` (class), `minecraft.multi_player_game_mode` (class), `minecraft.abstract_container_menu` (class), `minecraft.click_type` (class), `minecraft.inventory.selected` (field), `minecraft.abstract_container_menu.container_id` (field), `minecraft.inventory.get_item`, `minecraft.inventory.get_container_size`, `minecraft.item_stack.is_empty`, `minecraft.item_stack.get_count`, `minecraft.item_stack.get_item`, `minecraft.item.get_description_id`, `minecraft.multi_player_game_mode.handle_inventory_mouse_click` |

Identifiers are stable across provider renames: `minecraft.resource_location` is Mojang's
`ResourceLocation` in 1.21 and `Identifier` in 1.21.11, and `minecraft.resource_key.location` is
`location()` then `identifier()`. Inventory changes go through the game's own mechanism
(`handle_inventory_mouse_click`), not raw field writes. Network packets are out of v1.
