# Minecraft v1 symbol catalog

`data_version` 2 of the [bundled tables](../data/README.md) declares the symbols below: those
required by the mc-bridge API v1 catalog (`mc_bridge/spec/api-v1.md`, appendix), operation by
operation. Every [target configuration](configurations.md) resolves all the symbols of its
Minecraft version; no configuration has partial support.

The list is maintained as the `mc_mappings_generator` input
`consumers/mc_mappings/catalog-v1.json`, which names each symbol by its Mojang name and its
semantic types. Adding, removing or renaming a symbol means editing that file, regenerating the
tables and publishing a new `data_version`; published identifiers are never reused for another
meaning. Every identifier of `data_version` 1 (19 classes, 13 fields, 36 methods) is kept.

| | Classes | Fields | Methods |
| --- | --- | --- | --- |
| Catalog | 79 | 89 | 185 |
| Minecraft 1.21 | 77 | 89 | 175 |
| Minecraft 1.21.11 | 79 | 89 | 177 |

A member identifier is its class identifier, a dot and the name below
(`minecraft.entity` + `get_id` → `minecraft.entity.get_id`). ¹ exists in 1.21 only, ² in
1.21.11 only (`catalog.scoped`), ³ has a different type in the two versions.

| Class | Fields | Methods |
| --- | --- | --- |
| `minecraft.client` | `player`, `level`, `game_mode`, `game_renderer`, `hit_result`, `options`, `screen`, `right_click_delay`, `miss_time`, `timer`, `mouse_handler` | `get_instance`, `get_user`, `tick`, `get_connection`, `get_window` |
| `minecraft.blockable_event_loop` | — | `execute`, `is_same_thread` |
| `minecraft.user` | — | `get_name`, `get_profile_id` |
| `minecraft.entity` | `xo`, `yo`, `zo`, `tick_count`, `y_rot`, `x_rot`, `y_rot_o`, `x_rot_o`, `fall_distance` ³ | `get_id`, `get_uuid`, `get_type`, `get_x`, `get_y`, `get_z`, `get_y_rot`, `get_x_rot`, `set_y_rot`, `set_x_rot`, `get_delta_movement`, `set_delta_movement`, `is_alive`, `is_removed`, `on_ground`, `get_bounding_box`, `get_eye_height`, `is_shift_key_down`, `is_sprinting`, `is_swimming`, `is_invisible`, `is_currently_glowing`, `get_custom_name`, `get_vehicle`, `set_pos`, `set_on_ground`, `get_position_codec` |
| `minecraft.entity_type` | — | `get_description_id` |
| `minecraft.living_entity` | `hurt_time` | `get_health`, `get_max_health`, `is_fall_flying`, `get_item_by_slot`, `get_absorption_amount`, `is_using_item`, `get_use_item_remaining_ticks`, `get_main_hand_item`, `get_active_effects`, `get_attributes`, `swing` |
| `minecraft.player` | `container_menu` | `get_inventory`, `get_game_profile`, `get_food_data`, `get_abilities`, `get_attack_strength_scale`, `get_cooldowns` |
| `minecraft.local_player` | — | `send_position`, `set_sprinting`, `drop`, `respawn` |
| `minecraft.inventory` | `selected` | `get_item`, `get_container_size` |
| `minecraft.item_stack` | — | `is_empty`, `get_count`, `get_item`, `get_damage_value`, `get_max_damage`, `get_enchantments` |
| `minecraft.item` | — | `get_description_id` |
| `minecraft.level` | — | `dimension`, `get_block_state`, `clip` |
| `minecraft.client_level` | — | `entities_for_rendering`, `tick_rate_manager`, `get_entity` |
| `minecraft.resource_key` | — | `location` |
| `minecraft.resource_location` | — | — |
| `minecraft.vec3` | `x`, `y`, `z` | `new` |
| `minecraft.multi_player_game_mode` | `destroy_progress` | `handle_inventory_mouse_click`, `get_player_mode`, `is_destroying`, `attack`, `use_item`, `use_item_on`, `release_using_item`, `interact`, `start_destroy_block`, `continue_destroy_block`, `stop_destroy_block` |
| `minecraft.abstract_container_menu` | `container_id`, `slots` | `get_carried` |
| `minecraft.click_type` | `pickup`, `quick_move`, `swap`, `throw` | — |
| `minecraft.aabb` | `min_x`, `min_y`, `min_z`, `max_x`, `max_y`, `max_z` | — |
| `minecraft.component` | — | `get_string` |
| `minecraft.equipment_slot` | `mainhand`, `offhand`, `feet`, `legs`, `chest`, `head` | — |
| `minecraft.food_data` | — | `get_food_level`, `get_saturation_level` |
| `minecraft.game_type` | `survival`, `creative`, `adventure`, `spectator` | — |
| `minecraft.abilities` | `flying`, `mayfly`, `instabuild` | — |
| `minecraft.item_enchantments` | — | `entry_set` |
| `minecraft.holder` | — | `get_registered_name` |
| `minecraft.game_renderer` | — | `pick`, `get_main_camera`, `get_fov` ³ |
| `minecraft.hit_result` | — | `get_type`, `get_location` |
| `minecraft.hit_result_type` | `miss`, `block`, `entity` | — |
| `minecraft.block_hit_result` | — | `get_block_pos`, `get_direction`, `is_inside`, `new` |
| `minecraft.block_pos` | — | `new`, `get_x`, `get_y`, `get_z` |
| `minecraft.direction` | — | `get_3d_data_value`, `from_3d_data_value` |
| `minecraft.entity_hit_result` | — | `get_entity` |
| `minecraft.vec3i` | — | — |
| `minecraft.block_state` | — | `is_air`, `can_be_replaced`, `get_collision_shape`, `get_fluid_state`, `get_block` |
| `minecraft.block_state_base` | — | — |
| `minecraft.block_getter` | — | — |
| `minecraft.voxel_shape` | — | `is_empty` |
| `minecraft.fluid_state` | — | `is_empty` |
| `minecraft.block` | — | `get_description_id` |
| `minecraft.block_behaviour` | — | — |
| `minecraft.clip_context` | — | `new` |
| `minecraft.clip_context_block` | `collider`, `outline` | — |
| `minecraft.clip_context_fluid` | `none`, `any` | — |
| `minecraft.camera` | — | `get_position`, `get_y_rot`, `get_x_rot`, `is_detached` |
| `minecraft.options` | `key_attack`, `key_use`, `key_up`, `key_down`, `key_left`, `key_right`, `key_jump`, `key_shift`, `key_sprint`, `key_drop`, `key_swap_offhand`, `key_hotbar_slots` | `fov`, `gamma`, `sensitivity` |
| `minecraft.option_instance` | `value` | `get` |
| `minecraft.screen` | — | — |
| `minecraft.chat_screen` | — | — |
| `minecraft.inventory_screen` | — | — |
| `minecraft.creative_mode_inventory_screen` | — | — |
| `minecraft.abstract_container_screen` | — | — |
| `minecraft.non_null_list` | — | — |
| `minecraft.slot` | — | `get_item` |
| `minecraft.mob_effect_instance` | — | `get_effect`, `get_amplifier`, `get_duration` |
| `minecraft.attribute_map` | `attributes` | — |
| `minecraft.attribute_instance` | — | `get_attribute`, `get_base_value`, `get_value`, `set_base_value` |
| `minecraft.client_packet_listener` | — | `get_online_players`, `send_chat`, `send_command`, `send`, `handle_set_entity_motion`, `handle_teleport_entity`, `handle_move_entity`, `handle_entity_position_sync` ² |
| `minecraft.player_info` | — | `get_profile`, `get_latency`, `get_game_mode`, `get_tab_list_display_name` |
| `minecraft.key_mapping` | `click_count`, `key` | `is_down`, `set_down` |
| `minecraft.input_constants_key` | — | `get_value`, `get_type` |
| `minecraft.input_constants_type` | `mouse` | — |
| `minecraft.window` | — | `get_window` |
| `minecraft.item_cooldowns` | — | `get_cooldown_percent` ³ |
| `minecraft.delta_tracker_timer` | `ms_per_tick` | — |
| `minecraft.tick_rate_manager` | — | `tickrate`, `set_tick_rate` |
| `minecraft.mouse_handler` | `accumulated_dx`, `accumulated_dy` | `turn_player` |
| `minecraft.interaction_hand` | `main_hand`, `off_hand` | — |
| `minecraft.interaction_result` | — | — |
| `minecraft.client_common_packet_listener_impl` | — | — |
| `minecraft.packet` | — | — |
| `minecraft.set_entity_motion_packet` | — | `get_id`, `new`, `get_xa` ¹, `get_ya` ¹, `get_za` ¹, `get_movement` ² |
| `minecraft.teleport_entity_packet` | — | `get_id`, `is_on_ground`, `get_x` ¹, `get_y` ¹, `get_z` ¹, `get_y_rot` ¹, `get_x_rot` ¹, `change` ², `relatives` ² |
| `minecraft.move_entity_packet` | — | `get_entity`, `get_xa`, `get_ya`, `get_za`, `has_position`, `has_rotation`, `is_on_ground`, `get_y_rot` ³, `get_x_rot` ³ |
| `minecraft.vec_delta_codec` | — | `decode` |
| `minecraft.move_player_packet` | `x`, `y`, `z`, `y_rot`, `x_rot`, `on_ground`, `has_pos`, `has_rot` | — |
| `minecraft.position_move_rotation` ² | — | `position` ², `y_rot` ², `x_rot` ² |
| `minecraft.entity_position_sync_packet` ² | — | `id` ², `values` ², `on_ground` ² |

Identifiers are stable across provider renames: `minecraft.resource_location` is Mojang's
`ResourceLocation` in 1.21 and `Identifier` in 1.21.11, and `minecraft.resource_key.location` is
`location()` then `identifier()`. Mojang restructured the entity-motion and teleport packets
between the two versions: their content is read through version-scoped symbols.
