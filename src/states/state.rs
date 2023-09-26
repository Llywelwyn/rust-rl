use specs::prelude::*;
use bracket_lib::prelude::*;
use super::runstate::RunState;
use crate::map::*;
use crate::hunger_system;
use crate::particle_system;
use crate::trigger_system;
use crate::inventory;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::spatial;
use crate::effects;
use crate::visibility_system::VisibilitySystem;
use crate::ai;
use crate::gamelog;
use crate::spawner;
use crate::consts::ids::*;
use crate::consts::events::*;
use crate::components::*;
use crate::player::*;
use crate::gui;
use crate::config;
use crate::camera;
use crate::saveload_system;
use crate::morgue;
use crate::damage_system;
use std::collections::HashMap;
use notan::prelude::*;

#[derive(AppState)]
pub struct State {
    pub ecs: World,
    pub base_texture: Texture,
    pub atlas: HashMap<String, Texture>,
    pub font: notan::draw::Font,
    pub mapgen_next_state: Option<RunState>,
    pub mapgen_history: Vec<Map>,
    pub mapgen_index: usize,
    pub mapgen_timer: f32,
}

impl State {
    pub fn generate_world_map(&mut self, new_id: i32, dest_tile: TileType) {
        // Visualisation stuff
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let map_building_info = level_transition(&mut self.ecs, new_id, dest_tile);
        if let Some(history) = map_building_info {
            self.mapgen_history = history;
        } else {
            dungeon::thaw_entities(&mut self.ecs);
        }
    }

    fn run_systems(&mut self) {
        let mut hunger_clock = hunger_system::HungerSystem {};
        let mut particle_system = particle_system::ParticleSpawnSystem {};

        // Order is *very* important here, to ensure effects take place in the right order,
        // and that the player/AI are making decisions based on the most up-to-date info.

        self.resolve_entity_decisions(); //             Push Player messages of intent to effects queue, and run it.
        self.refresh_indexes(); //                      Get up-to-date map and viewsheds prior to AI turn.
        self.run_ai(); //                               Get AI decision-making.
        self.resolve_entity_decisions(); //             Push AI messages of intent to effects queue, and run it.
        hunger_clock.run_now(&self.ecs); //             Tick the hunger clock (on the turn clock's turn)
        particle_system.run_now(&self.ecs); //          Spawn/delete particles (turn independent)
        self.ecs.maintain();
    }

    fn resolve_entity_decisions(&mut self) {
        let mut trigger_system = trigger_system::TriggerSystem {};
        let mut inventory_system = inventory::ItemCollectionSystem {};
        let mut item_equip_system = inventory::ItemEquipSystem {};
        let mut item_use_system = inventory::ItemUseSystem {};
        let mut item_drop_system = inventory::ItemDropSystem {};
        let mut item_remove_system = inventory::ItemRemoveSystem {};
        let mut item_id_system = inventory::ItemIdentificationSystem {};
        let mut melee_system = MeleeCombatSystem {};
        trigger_system.run_now(&self.ecs);
        inventory_system.run_now(&self.ecs);
        item_equip_system.run_now(&self.ecs);
        item_use_system.run_now(&self.ecs);
        item_drop_system.run_now(&self.ecs);
        item_remove_system.run_now(&self.ecs);
        item_id_system.run_now(&self.ecs);
        melee_system.run_now(&self.ecs);

        effects::run_effects_queue(&mut self.ecs);
    }

    fn refresh_indexes(&mut self) {
        let mut mapindex = spatial::MapIndexingSystem {};
        let mut vis = VisibilitySystem {};
        mapindex.run_now(&self.ecs);
        vis.run_now(&self.ecs);
    }

    fn run_ai(&mut self) {
        let mut encumbrance_system = ai::EncumbranceSystem {}; // Must run first, as it affects energy regen.
        let mut energy = ai::EnergySystem {}; // Figures out who deserves a turn.
        let mut regen_system = ai::RegenSystem {}; // Restores HP on appropriate clock ticks.
        let mut turn_status_system = ai::TurnStatusSystem {}; // Ticks statuses. Should anyone now lose their turn? i.e. confusion
        let mut quip_system = ai::QuipSystem {}; // Quipping is "free". It doesn't use up a turn.
        let mut adjacent_ai = ai::AdjacentAI {}; // AdjacentAI -> DefaultAI are all exclusive. If one acts, the entity's turn is over.
        let mut visible_ai = ai::VisibleAI {};
        let mut approach_ai = ai::ApproachAI {};
        let mut flee_ai = ai::FleeAI {};
        let mut chase_ai = ai::ChaseAI {};
        let mut default_move_ai = ai::DefaultAI {};
        encumbrance_system.run_now(&self.ecs);
        energy.run_now(&self.ecs);
        regen_system.run_now(&self.ecs);
        turn_status_system.run_now(&self.ecs);
        quip_system.run_now(&self.ecs);
        adjacent_ai.run_now(&self.ecs);
        visible_ai.run_now(&self.ecs);
        approach_ai.run_now(&self.ecs);
        flee_ai.run_now(&self.ecs);
        chase_ai.run_now(&self.ecs);
        default_move_ai.run_now(&self.ecs);
    }

    fn goto_id(&mut self, id: i32, dest_tile: TileType) {
        // Freeze curr level
        dungeon::freeze_entities(&mut self.ecs);
        self.generate_world_map(id, dest_tile);
        let mapname = self.ecs.fetch::<Map>().name.clone();
        gamelog::Logger
            ::new()
            .append("You head to")
            .colour(rgb_to_u8(get_local_col(id)))
            .append_n(&mapname)
            .colour(WHITE)
            .period()
            .log();
        gamelog::record_event(EVENT::ChangedFloor(mapname));
    }

    fn game_over_cleanup(&mut self) {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        }

        // Spawn a new player and build new map
        {
            let player_entity = spawner::player(&mut self.ecs, 0, 0);
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = player_entity;
        }
        // Replace map list
        self.ecs.insert(dungeon::MasterDungeonMap::new());
        self.generate_world_map(1, TileType::Floor);

        gamelog::setup_log();
        gamelog::record_event(EVENT::Level(1));
    }
}

impl State {
    pub fn update(&mut self, ctx: &mut App) {
        let mut new_runstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            new_runstate = *runstate;
        }
        particle_system::particle_ticker(&mut self.ecs, ctx);
        match new_runstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                self.refresh_indexes();
                effects::run_effects_queue(&mut self.ecs);
                let mut can_act = false;
                {
                    let player_entity = self.ecs.fetch::<Entity>();
                    let turns = self.ecs.read_storage::<TakingTurn>();
                    if let Some(_) = turns.get(*player_entity) {
                        can_act = true;
                    }
                }
                if can_act {
                    let on_overmap = self.ecs.fetch::<Map>().overmap;
                    new_runstate = player_input(self, ctx, on_overmap);
                } else {
                    new_runstate = RunState::Ticking;
                }
            }
            RunState::Ticking => {
                while new_runstate == RunState::Ticking && particle_system::check_queue(&self.ecs) {
                    self.run_systems();
                    self.ecs.maintain();
                    try_spawn_interval(&mut self.ecs);
                    maybe_map_message(&mut self.ecs);
                    match *self.ecs.fetch::<RunState>() {
                        RunState::AwaitingInput => {
                            new_runstate = RunState::AwaitingInput;
                        }
                        RunState::MagicMapReveal { row, cursed } => {
                            new_runstate = RunState::MagicMapReveal { row, cursed };
                        }
                        RunState::ShowRemoveCurse => {
                            new_runstate = RunState::ShowRemoveCurse;
                        }
                        RunState::ShowIdentify => {
                            new_runstate = RunState::ShowIdentify;
                        }
                        _ => {
                            new_runstate = RunState::Ticking;
                        }
                    }
                }
            }
            RunState::Farlook { .. } => {
                let result = gui::show_farlook(self, ctx);
                match result {
                    gui::FarlookResult::NoResponse { x, y } => {
                        new_runstate = RunState::Farlook { x, y };
                    }
                    gui::FarlookResult::Cancel => {
                        new_runstate = RunState::AwaitingInput;
                    }
                }
            }
            RunState::ShowCheatMenu => {
                let result = gui::show_cheat_menu(self, ctx);
                match result {
                    gui::CheatMenuResult::Cancel => {
                        new_runstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::NoResponse => {}
                    gui::CheatMenuResult::Ascend => {
                        let id = self.ecs.fetch::<Map>().id - 1;
                        self.goto_id(id, TileType::DownStair);
                        self.mapgen_next_state = Some(RunState::PreRun);
                        new_runstate = RunState::MapGeneration;
                    }
                    gui::CheatMenuResult::Descend => {
                        let id = self.ecs.fetch::<Map>().id + 1;
                        self.goto_id(id, TileType::UpStair);
                        self.mapgen_next_state = Some(RunState::PreRun);
                        new_runstate = RunState::MapGeneration;
                    }
                    gui::CheatMenuResult::Heal => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let mut player_pools = pools.get_mut(*player).unwrap();
                        player_pools.hit_points.current = player_pools.hit_points.max;
                        new_runstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::MagicMap => {
                        let mut map = self.ecs.fetch_mut::<Map>();
                        for v in map.revealed_tiles.iter_mut() {
                            *v = true;
                        }
                        new_runstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::GodMode => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let mut player_pools = pools.get_mut(*player).unwrap();
                        gamelog::Logger::new().append("TOGGLED GOD MODE!").log();
                        player_pools.god = !player_pools.god;
                        new_runstate = RunState::AwaitingInput;
                    }
                }
            }
            // RunState::ShowInventory
            // RunState::ShowDropItem
            // RunState::ShowRemoveItem
            // RunState::ShowTargeting
            // RunState::ShowRemoveCurse
            // RunState::ShowIdentify
            RunState::ActionWithDirection { function } => {
                new_runstate = gui::get_input_direction(&mut self.ecs, ctx, function);
            }
            // RunState::MainMenu
            // RunState::CharacterCreation
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                new_runstate = RunState::MainMenu {
                    menu_selection: gui::MainMenuSelection::LoadGame,
                };
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                let write_to_morgue: Option<bool> = match result {
                    gui::YesNoResult::NoSelection => None,
                    gui::YesNoResult::No => Some(false),
                    gui::YesNoResult::Yes => Some(true),
                };
                if let Some(response) = write_to_morgue {
                    if response {
                        morgue::create_morgue_file(&self.ecs);
                    }
                    self.game_over_cleanup();
                    new_runstate = RunState::MapGeneration;
                    self.mapgen_next_state = Some(RunState::MainMenu {
                        menu_selection: gui::MainMenuSelection::NewGame,
                    });
                }
            }
            RunState::GoToLevel(id, dest_tile) => {
                self.goto_id(id, dest_tile);
                self.mapgen_next_state = Some(RunState::PreRun);
                new_runstate = RunState::MapGeneration;
            }
            // RunState::HelpScreen
            RunState::MagicMapReveal { row, cursed } => {
                let mut map = self.ecs.fetch_mut::<Map>();
                // Could probably toss this into a function somewhere, and/or
                // have multiple simple animations for it.
                for x in 0..map.width {
                    let idx;
                    if x % 2 == 0 {
                        idx = map.xy_idx(x as i32, row);
                    } else {
                        idx = map.xy_idx(x as i32, (map.height as i32) - 1 - row);
                    }
                    if !cursed {
                        map.revealed_tiles[idx] = true;
                    } else {
                        map.revealed_tiles[idx] = false;
                    }
                }
                // Dirtify viewshed only if cursed, so our currently visible tiles aren't removed too
                if cursed {
                    let player_entity = self.ecs.fetch::<Entity>();
                    let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
                    let viewshed = viewshed_components.get_mut(*player_entity);
                    if let Some(viewshed) = viewshed {
                        viewshed.dirty = true;
                    }
                }

                if (row as usize) == (map.height as usize) - 1 {
                    new_runstate = RunState::Ticking;
                } else {
                    new_runstate = RunState::MagicMapReveal { row: row + 1, cursed: cursed };
                }
            }
            RunState::MapGeneration => {
                if !config::CONFIG.logging.show_mapgen || self.mapgen_history.len() <= 0 {
                    new_runstate = self.mapgen_next_state.unwrap();
                } else {
                    if self.mapgen_history.len() > 0 {
                        console::log(
                            format!(
                                "mapgen_index: {} -- mapgen_history.len(): {} -- mapgen_timer: {} -- ctx.timer.delta_f32(): {}",
                                self.mapgen_index,
                                self.mapgen_history.len(),
                                self.mapgen_timer,
                                ctx.timer.delta_f32()
                            )
                        );
                        self.mapgen_timer += ctx.timer.delta_f32();
                        if
                            self.mapgen_timer > 10.0 / (self.mapgen_history.len() as f32) ||
                            self.mapgen_timer > 1.0
                        {
                            self.mapgen_timer = 0.0;
                            self.mapgen_index += 1;
                            if self.mapgen_index >= self.mapgen_history.len() {
                                new_runstate = self.mapgen_next_state.unwrap();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_runstate;
        }
        damage_system::delete_the_dead(&mut self.ecs);
    }

    // Deprecated
    fn tick(&mut self, ctx: &mut BTerm) {
        let mut new_runstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            new_runstate = *runstate;
        }
        // Clear screen
        ctx.set_active_console(2);
        ctx.cls();
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(0);
        ctx.cls();
        //particle_system::particle_ticker(&mut self.ecs, ctx);

        match new_runstate {
            RunState::MainMenu { .. } => {}
            RunState::CharacterCreation { .. } => {}
            _ => {
                // Draw map and ui
                camera::render_camera(&self.ecs, ctx);
                gui::draw_ui(&self.ecs, ctx);
            }
        }

        match new_runstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                // We refresh the index, and run anything that might
                // still be in the queue, just to make 100% sure that
                // there are no lingering effects from the last tick.
                self.refresh_indexes();
                effects::run_effects_queue(&mut self.ecs);
                // Sanity-checking that the player actually *should*
                // be taking a turn before giving them one. If they
                // don't have a turn component, go back to ticking.
                let mut can_act = false;
                {
                    let player_entity = self.ecs.fetch::<Entity>();
                    let turns = self.ecs.read_storage::<TakingTurn>();
                    if let Some(_) = turns.get(*player_entity) {
                        can_act = true;
                    }
                }
                if can_act {
                    let on_overmap = self.ecs.fetch::<Map>().overmap;
                    new_runstate = RunState::AwaitingInput; //player_input(self, ctx, on_overmap);
                } else {
                    new_runstate = RunState::Ticking;
                }
            }
            RunState::Ticking => {
                while new_runstate == RunState::Ticking && particle_system::check_queue(&self.ecs) {
                    self.run_systems();
                    self.ecs.maintain();
                    try_spawn_interval(&mut self.ecs);
                    maybe_map_message(&mut self.ecs);
                    match *self.ecs.fetch::<RunState>() {
                        RunState::AwaitingInput => {
                            new_runstate = RunState::AwaitingInput;
                        }
                        RunState::MagicMapReveal { row, cursed } => {
                            new_runstate = RunState::MagicMapReveal { row: row, cursed: cursed };
                        }
                        RunState::ShowRemoveCurse => {
                            new_runstate = RunState::ShowRemoveCurse;
                        }
                        RunState::ShowIdentify => {
                            new_runstate = RunState::ShowIdentify;
                        }
                        _ => {
                            new_runstate = RunState::Ticking;
                        }
                    }
                }
            }
            RunState::Farlook { .. } => {
                let result = gui::FarlookResult::Cancel; //gui::show_farlook(self, ctx);
                match result {
                    gui::FarlookResult::NoResponse { x, y } => {
                        new_runstate = RunState::Farlook { x, y };
                    }
                    gui::FarlookResult::Cancel => {
                        new_runstate = RunState::AwaitingInput;
                    }
                }
            }
            RunState::ShowCheatMenu => {
                let result = gui::CheatMenuResult::Cancel; //gui::show_cheat_menu(self, ctx);
                match result {
                    gui::CheatMenuResult::Cancel => {
                        new_runstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::NoResponse => {}
                    gui::CheatMenuResult::Ascend => {
                        let id = self.ecs.fetch::<Map>().id - 1;
                        self.goto_id(id, TileType::DownStair);
                        self.mapgen_next_state = Some(RunState::PreRun);
                        new_runstate = RunState::MapGeneration;
                    }
                    gui::CheatMenuResult::Descend => {
                        let id = self.ecs.fetch::<Map>().id + 1;
                        self.goto_id(id, TileType::UpStair);
                        self.mapgen_next_state = Some(RunState::PreRun);
                        new_runstate = RunState::MapGeneration;
                    }
                    gui::CheatMenuResult::Heal => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let mut player_pools = pools.get_mut(*player).unwrap();
                        player_pools.hit_points.current = player_pools.hit_points.max;
                        new_runstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::MagicMap => {
                        let mut map = self.ecs.fetch_mut::<Map>();
                        for v in map.revealed_tiles.iter_mut() {
                            *v = true;
                        }
                        new_runstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::GodMode => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let mut player_pools = pools.get_mut(*player).unwrap();
                        gamelog::Logger::new().append("TOGGLED GOD MODE!").log();
                        player_pools.god = !player_pools.god;
                        new_runstate = RunState::AwaitingInput;
                    }
                }
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => {
                        new_runstate = RunState::AwaitingInput;
                    }
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let ranged_item = is_ranged.get(item_entity);
                        if let Some(ranged_item) = ranged_item {
                            let is_aoe = self.ecs.read_storage::<AOE>();
                            let aoe_item = is_aoe.get(item_entity);
                            let bounds = camera::get_screen_bounds(&self.ecs, false);
                            let ppos = self.ecs.fetch::<Point>();
                            if let Some(aoe_item) = aoe_item {
                                new_runstate = RunState::ShowTargeting {
                                    x: ppos.x + bounds.x_offset - bounds.min_x,
                                    y: ppos.y + bounds.y_offset - bounds.min_y,
                                    range: ranged_item.range,
                                    item: item_entity,
                                    aoe: aoe_item.radius,
                                };
                            } else {
                                new_runstate = RunState::ShowTargeting {
                                    x: ppos.x + bounds.x_offset - bounds.min_x,
                                    y: ppos.y + bounds.y_offset - bounds.min_y,
                                    range: ranged_item.range,
                                    item: item_entity,
                                    aoe: 0,
                                };
                            }
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(*self.ecs.fetch::<Entity>(), WantsToUseItem {
                                    item: item_entity,
                                    target: None,
                                })
                                .expect("Unable to insert intent.");
                            new_runstate = RunState::Ticking;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => {
                        new_runstate = RunState::AwaitingInput;
                    }
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(*self.ecs.fetch::<Entity>(), WantsToDropItem {
                                item: item_entity,
                            })
                            .expect("Unable to insert intent");
                        new_runstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowRemoveItem => {
                let result = gui::remove_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => {
                        new_runstate = RunState::AwaitingInput;
                    }
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveItem>();
                        intent
                            .insert(*self.ecs.fetch::<Entity>(), WantsToRemoveItem {
                                item: item_entity,
                            })
                            .expect("Unable to insert intent");
                        new_runstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowTargeting { x, y, range, item, aoe } => {
                let result = gui::ranged_target(self, ctx, x, y, range, aoe);
                match result.0 {
                    gui::TargetResult::Cancel => {
                        new_runstate = RunState::AwaitingInput;
                    }
                    gui::TargetResult::NoResponse { x, y } => {
                        new_runstate = RunState::ShowTargeting { x, y, range, item, aoe };
                    }
                    gui::TargetResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent
                            .insert(*self.ecs.fetch::<Entity>(), WantsToUseItem {
                                item,
                                target: result.1,
                            })
                            .expect("Unable to insert intent.");
                        new_runstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowRemoveCurse => {
                let result = gui::remove_curse(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => {
                        new_runstate = RunState::AwaitingInput;
                    }
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        self.ecs
                            .write_storage::<Beatitude>()
                            .insert(item_entity, Beatitude { buc: BUC::Uncursed, known: true })
                            .expect("Unable to insert beatitude");
                        new_runstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowIdentify => {
                let result = gui::identify(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => {
                        new_runstate = RunState::AwaitingInput;
                    }
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        if let Some(name) = self.ecs.read_storage::<Name>().get(item_entity) {
                            let mut dm = self.ecs.fetch_mut::<MasterDungeonMap>();
                            dm.identified_items.insert(name.name.clone());
                        }
                        if
                            let Some(beatitude) = self.ecs
                                .write_storage::<Beatitude>()
                                .get_mut(item_entity)
                        {
                            beatitude.known = true;
                        }
                        new_runstate = RunState::Ticking;
                    }
                }
            }
            RunState::ActionWithDirection { function } => {
                new_runstate = RunState::AwaitingInput; //gui::get_input_direction(&mut self.ecs, ctx, function);
            }
            RunState::MainMenu { .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection { selected } => {
                        new_runstate = RunState::MainMenu { menu_selection: selected };
                    }
                    gui::MainMenuResult::Selected { selected } =>
                        match selected {
                            gui::MainMenuSelection::NewGame => {
                                new_runstate = RunState::CharacterCreation {
                                    ancestry: gui::Ancestry::Human,
                                    class: gui::Class::Fighter,
                                };
                            }
                            gui::MainMenuSelection::LoadGame => {
                                saveload_system::load_game(&mut self.ecs);
                                new_runstate = RunState::AwaitingInput;
                                saveload_system::delete_save();
                            }
                            gui::MainMenuSelection::Quit => {
                                ::std::process::exit(0);
                            }
                        }
                }
            }
            RunState::CharacterCreation { .. } => {
                let result = gui::character_creation(self, ctx);
                match result {
                    gui::CharCreateResult::NoSelection { ancestry, class } => {
                        new_runstate = RunState::CharacterCreation { ancestry, class };
                    }
                    gui::CharCreateResult::Selected { ancestry, class } => {
                        if ancestry == gui::Ancestry::Unset {
                            new_runstate = RunState::MainMenu {
                                menu_selection: gui::MainMenuSelection::NewGame,
                            };
                        } else {
                            gui::setup_player_ancestry(&mut self.ecs, ancestry);
                            gui::setup_player_class(&mut self.ecs, class, ancestry);
                            new_runstate = RunState::PreRun;
                        }
                    }
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                new_runstate = RunState::MainMenu {
                    menu_selection: gui::MainMenuSelection::LoadGame,
                };
            }
            RunState::GameOver => {
                let result = gui::YesNoResult::No; //gui::game_over(ctx);
                let write_to_morgue: Option<bool> = match result {
                    gui::YesNoResult::NoSelection => None,
                    gui::YesNoResult::No => Some(false),
                    gui::YesNoResult::Yes => Some(true),
                };
                if let Some(response) = write_to_morgue {
                    if response {
                        morgue::create_morgue_file(&self.ecs);
                    }
                    self.game_over_cleanup();
                    new_runstate = RunState::MapGeneration;
                    self.mapgen_next_state = Some(RunState::MainMenu {
                        menu_selection: gui::MainMenuSelection::NewGame,
                    });
                }
            }
            RunState::GoToLevel(id, dest_tile) => {
                self.goto_id(id, dest_tile);
                self.mapgen_next_state = Some(RunState::PreRun);
                new_runstate = RunState::MapGeneration;
            }
            RunState::HelpScreen => {
                let result = gui::show_help(ctx);
                match result {
                    gui::YesNoResult::Yes => {
                        gamelog::record_event(EVENT::LookedForHelp(1));
                        new_runstate = RunState::AwaitingInput;
                    }
                    _ => {}
                }
            }
            RunState::MagicMapReveal { row, cursed } => {
                let mut map = self.ecs.fetch_mut::<Map>();

                // Could probably toss this into a function somewhere, and/or
                // have multiple simple animations for it.
                for x in 0..map.width {
                    let idx;
                    if x % 2 == 0 {
                        idx = map.xy_idx(x as i32, row);
                    } else {
                        idx = map.xy_idx(x as i32, (map.height as i32) - 1 - row);
                    }
                    if !cursed {
                        map.revealed_tiles[idx] = true;
                    } else {
                        map.revealed_tiles[idx] = false;
                    }
                }

                // Dirtify viewshed only if cursed, so our currently visible tiles aren't removed too
                if cursed {
                    let player_entity = self.ecs.fetch::<Entity>();
                    let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
                    let viewshed = viewshed_components.get_mut(*player_entity);
                    if let Some(viewshed) = viewshed {
                        viewshed.dirty = true;
                    }
                }

                if (row as usize) == (map.height as usize) - 1 {
                    new_runstate = RunState::Ticking;
                } else {
                    new_runstate = RunState::MagicMapReveal { row: row + 1, cursed: cursed };
                }
            }
            RunState::MapGeneration => {
                if !config::CONFIG.logging.show_mapgen {
                    new_runstate = self.mapgen_next_state.unwrap();
                }
                if self.mapgen_history.len() != 0 {
                    ctx.set_active_console(2);
                    ctx.cls();
                    ctx.set_active_console(1);
                    ctx.cls();
                    ctx.set_active_console(0);
                    ctx.cls();
                    camera::render_debug_map(&self.mapgen_history[self.mapgen_index], ctx);

                    self.mapgen_timer += ctx.frame_time_ms;
                    if self.mapgen_timer > 300.0 {
                        self.mapgen_timer = 0.0;
                        self.mapgen_index += 1;
                        if self.mapgen_index >= self.mapgen_history.len() {
                            new_runstate = self.mapgen_next_state.unwrap();
                        }
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_runstate;
        }

        damage_system::delete_the_dead(&mut self.ecs);

        let _ = render_draw_buffer(ctx);
    }
}
