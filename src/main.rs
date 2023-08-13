use rltk::{GameState, Point, RandomNumberGenerator, Rltk};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
extern crate serde;

pub mod camera;
mod components;
pub mod raws;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod gamelog;
mod gui;
pub mod map_builders;
mod saveload_system;
mod spawner;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai_system;
use monster_ai_system::MonsterAI;
pub mod bystander_ai_system;
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod damage_system;
use damage_system::*;
mod hunger_system;
mod melee_combat_system;
mod trigger_system;
use melee_combat_system::MeleeCombatSystem;
mod inventory_system;
use inventory_system::*;
mod particle_system;
use particle_system::{ParticleBuilder, DEFAULT_PARTICLE_LIFETIME, LONG_PARTICLE_LIFETIME};
mod ai;
mod gamesystem;
mod random_table;
mod rex_assets;

#[macro_use]
extern crate lazy_static;

//Consts
pub const SHOW_MAPGEN: bool = false;
pub const LOG_SPAWNING: bool = true;
pub const LOG_TICKS: bool = false;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput, // Player's turn
    PreRun,
    Ticking,       // Tick systems
    ShowCheatMenu, // Teleport, godmode, etc. - for debugging
    ShowInventory,
    ShowDropItem,
    ShowRemoveItem,
    ShowTargeting { range: i32, item: Entity, aoe: i32 },
    ActionWithDirection { function: fn(i: i32, j: i32, ecs: &mut World) -> RunState },
    MainMenu { menu_selection: gui::MainMenuSelection },
    SaveGame,
    GameOver,
    NextLevel,
    PreviousLevel,
    HelpScreen,
    MagicMapReveal { row: i32, cursed: bool }, // Animates magic mapping effect
    MapGeneration,
}

pub struct State {
    pub ecs: World,
    mapgen_next_state: Option<RunState>,
    mapgen_history: Vec<Map>,
    mapgen_index: usize,
    mapgen_timer: f32,
}

impl State {
    fn generate_world_map(&mut self, new_id: i32, offset: i32) {
        // Visualisation stuff
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let map_building_info = map::level_transition(&mut self.ecs, new_id, offset);
        if let Some(history) = map_building_info {
            self.mapgen_history = history;
        } else {
            map::dungeon::thaw_entities(&mut self.ecs);
        }
    }

    fn run_systems(&mut self) {
        let mut mapindex = MapIndexingSystem {};
        let mut vis = VisibilitySystem {};
        let mut regen_system = ai::RegenSystem {};
        let mut energy = ai::EnergySystem {};
        let mut encumbrance_system = ai::EncumbranceSystem {};
        let mut turn_status_system = ai::TurnStatusSystem {};
        let mut quip_system = ai::QuipSystem {};
        let mut mob = MonsterAI {};
        let mut bystanders = bystander_ai_system::BystanderAI {};
        let mut trigger_system = trigger_system::TriggerSystem {};
        let mut melee_system = MeleeCombatSystem {};
        let mut damage_system = DamageSystem {};
        let mut inventory_system = ItemCollectionSystem {};
        let mut item_use_system = ItemUseSystem {};
        let mut item_drop_system = ItemDropSystem {};
        let mut item_remove_system = ItemRemoveSystem {};
        let mut hunger_clock = hunger_system::HungerSystem {};
        let mut particle_system = particle_system::ParticleSpawnSystem {};

        mapindex.run_now(&self.ecs);
        vis.run_now(&self.ecs);
        regen_system.run_now(&self.ecs);
        encumbrance_system.run_now(&self.ecs);
        energy.run_now(&self.ecs);
        turn_status_system.run_now(&self.ecs);
        quip_system.run_now(&self.ecs);
        mob.run_now(&self.ecs);
        bystanders.run_now(&self.ecs);
        trigger_system.run_now(&self.ecs);
        inventory_system.run_now(&self.ecs);
        item_use_system.run_now(&self.ecs);
        item_drop_system.run_now(&self.ecs);
        item_remove_system.run_now(&self.ecs);
        melee_system.run_now(&self.ecs);
        damage_system.run_now(&self.ecs);
        hunger_clock.run_now(&self.ecs);
        particle_system.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn run_map_index(&mut self) {
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
    }

    fn goto_level(&mut self, offset: i32) {
        // Build new map + place player
        let current_id;
        {
            let worldmap_resource = self.ecs.fetch::<Map>();
            current_id = worldmap_resource.id;
        }
        // Record the correct type of event
        if offset > 0 {
            gamelog::record_event("descended", 1);
        } else if current_id == 1 {
            gamelog::Logger::new().append("CHEAT MENU: YOU CAN'T DO THAT.").colour((255, 0, 0)).log();
            return;
        } else {
            gamelog::record_event("ascended", 1);
        }
        // Freeze the current level
        map::dungeon::freeze_entities(&mut self.ecs);
        self.generate_world_map(current_id + offset, offset);
        let mapname = self.ecs.fetch::<Map>().name.clone();
        gamelog::Logger::new().append("You head to").npc_name_n(mapname).period().log();
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
        self.ecs.insert(map::dungeon::MasterDungeonMap::new());
        self.generate_world_map(1, 0);

        gamelog::setup_log();
        gamelog::record_event("player_level", 1);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut new_runstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            new_runstate = *runstate;
        }
        // Clear screen
        ctx.cls();
        particle_system::particle_ticker(&mut self.ecs, ctx);

        match new_runstate {
            RunState::MainMenu { .. } => {}
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
                self.run_map_index();
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
                    new_runstate = player_input(self, ctx);
                } else {
                    new_runstate = RunState::Ticking;
                }
            }
            RunState::Ticking => {
                while new_runstate == RunState::Ticking {
                    self.run_systems();
                    self.ecs.maintain();
                    try_spawn_interval(&mut self.ecs);
                    match *self.ecs.fetch::<RunState>() {
                        RunState::AwaitingInput => new_runstate = RunState::AwaitingInput,
                        RunState::MagicMapReveal { row, cursed } => {
                            new_runstate = RunState::MagicMapReveal { row: row, cursed: cursed }
                        }
                        _ => new_runstate = RunState::Ticking,
                    }
                }
            }
            RunState::ShowCheatMenu => {
                let result = gui::show_cheat_menu(self, ctx);
                match result {
                    gui::CheatMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::CheatMenuResult::NoResponse => {}
                    gui::CheatMenuResult::Ascend => {
                        self.goto_level(-1);
                        self.mapgen_next_state = Some(RunState::PreRun);
                        new_runstate = RunState::MapGeneration;
                    }
                    gui::CheatMenuResult::Descend => {
                        self.goto_level(1);
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
                        gamelog::Logger::new().item_name("TOGGLED GOD MODE!").log();
                        player_pools.god = !player_pools.god;
                        new_runstate = RunState::AwaitingInput;
                    }
                }
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let ranged_item = is_ranged.get(item_entity);
                        if let Some(ranged_item) = ranged_item {
                            let is_aoe = self.ecs.read_storage::<AOE>();
                            let aoe_item = is_aoe.get(item_entity);
                            if let Some(aoe_item) = aoe_item {
                                new_runstate = RunState::ShowTargeting {
                                    range: ranged_item.range,
                                    item: item_entity,
                                    aoe: aoe_item.radius,
                                }
                            } else {
                                new_runstate =
                                    RunState::ShowTargeting { range: ranged_item.range, item: item_entity, aoe: 0 }
                            }
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item: item_entity, target: None })
                                .expect("Unable to insert intent.");
                            new_runstate = RunState::Ticking;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(*self.ecs.fetch::<Entity>(), WantsToDropItem { item: item_entity })
                            .expect("Unable to insert intent");
                        new_runstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowRemoveItem => {
                let result = gui::remove_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveItem>();
                        intent
                            .insert(*self.ecs.fetch::<Entity>(), WantsToRemoveItem { item: item_entity })
                            .expect("Unable to insert intent");
                        new_runstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowTargeting { range, item, aoe } => {
                let result = gui::ranged_target(self, ctx, range, aoe);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent
                            .insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item, target: result.1 })
                            .expect("Unable to insert intent.");
                        new_runstate = RunState::Ticking;
                    }
                }
            }
            RunState::ActionWithDirection { function } => {
                new_runstate = gui::get_input_direction(&mut self.ecs, ctx, function);
            }
            RunState::MainMenu { .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection { selected } => {
                        new_runstate = RunState::MainMenu { menu_selection: selected }
                    }
                    gui::MainMenuResult::Selected { selected } => match selected {
                        gui::MainMenuSelection::NewGame => new_runstate = RunState::PreRun,
                        gui::MainMenuSelection::LoadGame => {
                            saveload_system::load_game(&mut self.ecs);
                            new_runstate = RunState::AwaitingInput;
                            saveload_system::delete_save();
                        }
                        gui::MainMenuSelection::Quit => {
                            ::std::process::exit(0);
                        }
                    },
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                new_runstate = RunState::MainMenu { menu_selection: gui::MainMenuSelection::LoadGame };
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::YesNoResult::NoSelection => {}
                    gui::YesNoResult::Yes => {
                        self.game_over_cleanup();
                        new_runstate = RunState::MapGeneration;
                        self.mapgen_next_state =
                            Some(RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame });
                    }
                }
            }
            RunState::NextLevel => {
                self.goto_level(1);
                self.mapgen_next_state = Some(RunState::PreRun);
                new_runstate = RunState::MapGeneration;
            }
            RunState::PreviousLevel => {
                self.goto_level(-1);
                self.mapgen_next_state = Some(RunState::PreRun);
                new_runstate = RunState::MapGeneration;
            }
            RunState::HelpScreen => {
                let result = gui::show_help(ctx);
                match result {
                    gui::YesNoResult::NoSelection => {}
                    gui::YesNoResult::Yes => {
                        gamelog::record_event("looked_for_help", 1);
                        new_runstate = RunState::AwaitingInput;
                    }
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
                        idx = map.xy_idx(x as i32, (map.height as i32 - 1) - (row));
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

                if row as usize == map.height as usize - 1 {
                    new_runstate = RunState::Ticking;
                } else {
                    new_runstate = RunState::MagicMapReveal { row: row + 1, cursed: cursed };
                }
            }
            RunState::MapGeneration => {
                if !SHOW_MAPGEN {
                    new_runstate = self.mapgen_next_state.unwrap();
                }
                if self.mapgen_history.len() != 0 {
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

        let _ = rltk::render_draw_buffer(ctx);
    }
}

const DISPLAYWIDTH: i32 = 105;
const DISPLAYHEIGHT: i32 = 56;

fn main() -> rltk::BError {
    // Embedded resources for use in wasm build
    const CURSES_14_16_BYTES: &[u8] = include_bytes!("../resources/curses14x16.png");
    rltk::embedding::EMBED.lock().add_resource("resources/curses14x16.png".to_string(), CURSES_14_16_BYTES);

    //rltk::link_resource!(CURSES14X16, "../resources/curses_14x16.png");

    use rltk::RltkBuilder;
    let context = RltkBuilder::new()
        .with_title("rust-rl")
        .with_dimensions(DISPLAYWIDTH, DISPLAYHEIGHT)
        .with_font("curses14x16.png", 14, 16)
        .with_tile_dimensions(14, 16)
        .with_simple_console(DISPLAYWIDTH, DISPLAYHEIGHT, "curses14x16.png")
        //.with_simple_console_no_bg(DISPLAYWIDTH, DISPLAYHEIGHT, "terminal8x8.jpg")
        .build()?;

    let mut gs = State {
        ecs: World::new(),
        mapgen_next_state: Some(RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame }),
        mapgen_index: 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
    };

    gs.ecs.register::<Position>();
    gs.ecs.register::<OtherLevelPosition>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Burden>();
    gs.ecs.register::<Prop>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Clock>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Bystander>();
    gs.ecs.register::<Quips>();
    gs.ecs.register::<Mind>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Telepath>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<ObfuscatedName>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<BlocksVisibility>();
    gs.ecs.register::<Door>();
    gs.ecs.register::<Pools>();
    gs.ecs.register::<Attributes>();
    gs.ecs.register::<Skills>();
    gs.ecs.register::<HungerClock>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<MagicItem>();
    gs.ecs.register::<GrantsXP>();
    gs.ecs.register::<LootTable>();
    gs.ecs.register::<Energy>();
    gs.ecs.register::<TakingTurn>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<EquipmentChanged>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<MeleeWeapon>();
    gs.ecs.register::<NaturalAttacks>();
    gs.ecs.register::<ArmourClassBonus>();
    gs.ecs.register::<Cursed>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<AOE>();
    gs.ecs.register::<Digger>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<MagicMapper>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToRemoveItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<SingleActivation>();
    gs.ecs.register::<Wand>();
    gs.ecs.register::<ProvidesNutrition>();
    gs.ecs.register::<Destructible>();
    gs.ecs.register::<Hidden>();
    gs.ecs.register::<EntryTrigger>();
    gs.ecs.register::<EntityMoved>();
    gs.ecs.register::<MultiAttack>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<DMSerializationHelper>();
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    raws::load_raws();

    // Insert calls
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    gs.ecs.insert(map::MasterDungeonMap::new()); // Master map list
    gs.ecs.insert(Map::new(1, 64, 64, 0, "New Map")); // Map
    gs.ecs.insert(Point::new(0, 0)); // Player pos
    let player_entity = spawner::player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(player_entity); // Player entity
    gs.ecs.insert(RunState::MapGeneration {}); // RunState
    gs.ecs.insert(particle_system::ParticleBuilder::new());
    gs.ecs.insert(rex_assets::RexAssets::new());

    gamelog::setup_log();
    gamelog::record_event("player_level", 1);
    gs.generate_world_map(1, 0);

    rltk::main_loop(context, gs)
}
