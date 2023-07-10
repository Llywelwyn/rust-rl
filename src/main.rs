use rltk::{GameState, Point, Rltk, RGB};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
use std::ops::Add;
extern crate serde;

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod gamelog;
mod gui;
mod saveload_system;
mod spawner;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai_system;
use monster_ai_system::MonsterAI;
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod damage_system;
use damage_system::*;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod inventory_system;
use inventory_system::*;
mod particle_system;
use particle_system::{ParticleBuilder, DEFAULT_PARTICLE_LIFETIME, LONG_PARTICLE_LIFETIME};
mod rex_assets;

// Embedded resources for use in wasm build
rltk::embedded_resource!(TERMINAL8X8, "../resources/terminal8x8.jpg");
rltk::embedded_resource!(SCANLINESFS, "../resources/scanlines.fs");
rltk::embedded_resource!(SCANLINESVS, "../resources/scanlines.vs");

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: i32, item: Entity, aoe: i32 },
    MainMenu { menu_selection: gui::MainMenuSelection },
    SaveGame,
    NextLevel,
}

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut inventory_system = ItemCollectionSystem {};
        inventory_system.run_now(&self.ecs);
        let mut item_use_system = ItemUseSystem {};
        item_use_system.run_now(&self.ecs);
        let mut drop_system = ItemDropSystem {};
        drop_system.run_now(&self.ecs);
        let mut melee_system = MeleeCombatSystem {};
        melee_system.run_now(&self.ecs);
        let mut damage_system = DamageSystem {};
        damage_system.run_now(&self.ecs);
        let mut particle_system = particle_system::ParticleSpawnSystem {};
        particle_system.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let player_entity = self.ecs.fetch::<Entity>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for entity in entities.join() {
            let mut should_delete = true;

            // Don't delete player
            let p = player.get(entity);
            if let Some(_p) = p {
                should_delete = false;
            }

            // Don't delete player's equipment
            let bp = backpack.get(entity);
            if let Some(bp) = bp {
                if bp.owner == *player_entity {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }
        }

        return to_delete;
    }

    fn goto_next_level(&mut self) {
        // Delete entities that aren't player/player's equipment
        let to_delete = self.entities_to_remove_on_level_change();
        for target in to_delete {
            self.ecs.delete_entity(target).expect("Unable to delete entity");
        }

        // Build new map
        let worldmap;
        {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            let current_depth = worldmap_resource.depth;
            *worldmap_resource = Map::new_map_rooms_and_corridors(current_depth + 1);
            worldmap = worldmap_resource.clone();
        }

        // Spawn things in rooms
        for room in worldmap.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, room);
        }

        // Place the player and update resources
        let (player_x, player_y) = worldmap.rooms[0].centre();
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = Point::new(player_x, player_y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        let player_pos_comp = position_components.get_mut(*player_entity);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }

        // Dirtify viewshed
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        let viewshed = viewshed_components.get_mut(*player_entity);
        if let Some(viewshed) = viewshed {
            viewshed.dirty = true;
        }

        // Notify player, restore health up to a point.
        let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
        gamelog.entries.push("You descend the stairwell, and take a moment to recover your strength.".to_string());
        let mut player_health_store = self.ecs.write_storage::<CombatStats>();
        let player_health = player_health_store.get_mut(*player_entity);
        if let Some(player_health) = player_health {
            player_health.hp = i32::max(player_health.hp, player_health.max_hp / 2);
        }
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
        particle_system::cull_dead_particles(&mut self.ecs, ctx);

        match new_runstate {
            RunState::MainMenu { .. } => {}
            _ => {
                // Draw map and ui
                draw_map(&self.ecs, ctx);
                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let map = self.ecs.fetch::<Map>();

                    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
                    for (pos, render) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        let offsets = RGB::from_u8(map.red_offset[idx], map.green_offset[idx], map.blue_offset[idx]);
                        let mut bg = render.bg.add(RGB::from_u8(26, 45, 45)).add(offsets);
                        //bg = bg.add(offsets);
                        if map.bloodstains.contains(&idx) {
                            bg = bg.add(RGB::from_f32(0.6, 0., 0.));
                        }
                        if map.visible_tiles[idx] {
                            ctx.set(pos.x, pos.y, render.fg, bg, render.glyph);
                        }
                    }
                    gui::draw_ui(&self.ecs, ctx);
                }
            }
        }

        match new_runstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_runstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                new_runstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                new_runstate = RunState::AwaitingInput;
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
                            new_runstate = RunState::PlayerTurn;
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
                        new_runstate = RunState::PlayerTurn;
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
                        new_runstate = RunState::PlayerTurn;
                    }
                }
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
            RunState::NextLevel => {
                self.goto_next_level();
                new_runstate = RunState::PreRun;
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_runstate;
        }

        damage_system::delete_the_dead(&mut self.ecs);
    }
}

const DISPLAYWIDTH: i32 = 80;
const DISPLAYHEIGHT: i32 = 50;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::new()
        .with_title("rust-rl")
        .with_dimensions(DISPLAYWIDTH, DISPLAYHEIGHT)
        .with_fullscreen(true)
        .with_tile_dimensions(16, 16)
        .with_resource_path("resources/")
        .with_font("terminal8x8.jpg", 8, 8)
        .with_simple_console(DISPLAYWIDTH, DISPLAYHEIGHT, "terminal8x8.jpg")
        .with_simple_console_no_bg(DISPLAYWIDTH, DISPLAYHEIGHT, "terminal8x8.jpg")
        .build()?;
    context.with_post_scanlines(false);
    //context.screen_burn_color(RGB::named((150, 255, 255)));
    let mut gs = State { ecs: World::new() };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<AOE>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<Destructible>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    let map = Map::new_map_rooms_and_corridors(1);
    let (player_x, player_y) = map.rooms[0].centre();
    let player_name = "wanderer".to_string();
    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y, player_name);

    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(player_entity);
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["<pretend i wrote a paragraph explaining why you're here>".to_string()],
    });
    gs.ecs.insert(RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame });
    gs.ecs.insert(particle_system::ParticleBuilder::new());
    gs.ecs.insert(rex_assets::RexAssets::new());

    rltk::main_loop(context, gs)
}
