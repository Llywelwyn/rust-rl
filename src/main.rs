use rust_rl::*;
use notan::prelude::*;
use notan::draw::create_textures_from_atlas;
use notan::draw::{ CreateFont, CreateDraw, DrawImages };
use specs::prelude::*;
use specs::saveload::{ SimpleMarker, SimpleMarkerAllocator };
use bracket_lib::prelude::*;
use std::collections::HashMap;

const TILESIZE: u32 = 16;
const DISPLAYWIDTH: u32 = 100 * TILESIZE;
const DISPLAYHEIGHT: u32 = 56 * TILESIZE;

#[notan_main]
fn main() -> Result<(), String> {
    let win_config = WindowConfig::new().set_size(DISPLAYWIDTH, DISPLAYHEIGHT).set_vsync(true);
    notan
        ::init_with(setup)
        .add_config(win_config)
        .add_config(notan::draw::DrawConfig)
        .draw(draw)
        .update(update)
        .build()
}

fn setup(gfx: &mut Graphics) -> State {
    let texture = gfx
        .create_texture()
        .from_image(include_bytes!("../resources/td.png"))
        .build()
        .unwrap();
    let data = include_bytes!("../resources/td.json");
    let atlas = create_textures_from_atlas(data, &texture).unwrap();

    let mut gs = State {
        ecs: World::new(),
        base_texture: texture,
        atlas,
        mapgen_next_state: Some(RunState::MainMenu {
            menu_selection: gui::MainMenuSelection::NewGame,
        }),
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
    gs.ecs.register::<HasAncestry>();
    gs.ecs.register::<HasClass>();
    gs.ecs.register::<Chasing>();
    gs.ecs.register::<Faction>();
    gs.ecs.register::<Clock>();
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
    gs.ecs.register::<Item>();
    gs.ecs.register::<Beatitude>();
    gs.ecs.register::<IdentifiedItem>();
    gs.ecs.register::<IdentifiedBeatitude>();
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
    gs.ecs.register::<ToHitBonus>();
    gs.ecs.register::<MoveMode>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<AOE>();
    gs.ecs.register::<Digger>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<Blind>();
    gs.ecs.register::<MagicMapper>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToApproach>();
    gs.ecs.register::<WantsToFlee>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToRemoveItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<SingleActivation>();
    gs.ecs.register::<Charges>();
    gs.ecs.register::<ProvidesNutrition>();
    gs.ecs.register::<Destructible>();
    gs.ecs.register::<Hidden>();
    gs.ecs.register::<EntryTrigger>();
    gs.ecs.register::<EntityMoved>();
    gs.ecs.register::<MultiAttack>();
    gs.ecs.register::<ProvidesRemoveCurse>();
    gs.ecs.register::<ProvidesIdentify>();
    gs.ecs.register::<KnownSpells>();
    gs.ecs.register::<GrantsSpell>();
    gs.ecs.register::<Bleeds>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<SpawnParticleSimple>();
    gs.ecs.register::<SpawnParticleBurst>();
    gs.ecs.register::<SpawnParticleLine>();
    gs.ecs.register::<HasDamageModifiers>();
    gs.ecs.register::<Intrinsics>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<DMSerializationHelper>();
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    raws::load_raws();

    // Insert calls
    gs.ecs.insert(RandomNumberGenerator::new());
    gs.ecs.insert(map::MasterDungeonMap::new()); // Master map list
    gs.ecs.insert(Map::new(true, 1, 64, 64, 0, "New Map", "N", 0)); // Map
    gs.ecs.insert(Point::new(0, 0)); // Player pos
    gs.ecs.insert(gui::Ancestry::Human); // ancestry
    let player_entity = spawner::player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(player_entity); // Player entity
    gs.ecs.insert(RunState::AwaitingInput {}); // TODO: Set this back to RunState::MapGen
    gs.ecs.insert(particle_system::ParticleBuilder::new());
    gs.ecs.insert(rex_assets::RexAssets::new());

    gamelog::setup_log();
    gamelog::record_event(data::events::EVENT::Level(1));
    gs.generate_world_map(1, TileType::Floor);

    gs
}
const ASCII_MODE: bool = false; // Change this to config setting
const SHOW_BOUNDARIES: bool = false; // Config setting
use notan::draw::Draw;
fn draw_camera(ecs: &World, draw: &mut Draw, atlas: &HashMap<String, Texture>) {
    let map = ecs.fetch::<Map>();
    let bounds = crate::camera::get_screen_bounds(ecs);
    render_map_in_view(&*map, ecs, draw, bounds);
}

use crate::camera::ScreenBounds;
fn render_map_in_view(map: &Map, ecs: &World, draw: &mut Draw, bounds: ScreenBounds) {
    for tile_y in bounds.min_y..bounds.max_y {
        for tile_x in bounds.min_x..bounds.max_x {
            if crate::camera::in_bounds(tile_x, tile_y, map.width, map.height) {
                let idx = map.xy_idx(tile_x, tile_y);
                if map.revealed_tiles[idx] {
                    if ASCII_MODE {
                        let (glyph, fg, bg) = crate::map::themes::get_tile_renderables_for_id(
                            idx,
                            &*map,
                            Some(*ecs.fetch::<Point>()),
                            None
                        );
                        //  TODO: Draw ASCII
                    } else {
                        let (id, tint) = crate::map::themes::get_sprite_for_id(
                            idx,
                            &*map,
                            Some(*ecs.fetch::<Point>())
                        );
                        let px = idx_to_px(
                            map.xy_idx(tile_x + bounds.x_offset, tile_y + bounds.y_offset),
                            &map
                        );
                        draw.image(atlas.get(id).unwrap()).position(px.0, px.1);
                    }
                }
            } else if SHOW_BOUNDARIES {
                // TODO: Draw boundaries
            }
        }
    }
}

fn draw(app: &mut App, gfx: &mut Graphics, gs: &mut State) {
    let mut draw = gfx.create_draw();
    draw.clear(Color::BLACK);
    // Draw map
    draw_camera(&gs.ecs, &mut draw, &gs.atlas);
    // Draw player (replace this with draw entities).
    let map = gs.ecs.fetch::<Map>();
    let ppos = gs.ecs.fetch::<Point>();
    let offsets = crate::camera::get_offset();
    let px = idx_to_px(map.xy_idx(ppos.x + offsets.0, ppos.y + offsets.1), &map);
    draw.image(gs.atlas.get("ui_heart_full").unwrap()).position(px.0, px.1);
    // Render batch
    gfx.render(&draw);
}

fn idx_to_px(idx: usize, map: &Map) -> (f32, f32) {
    (
        ((idx % (map.width as usize)) as i32 as f32) * (TILESIZE as f32),
        ((idx / (map.width as usize)) as i32 as f32) * (TILESIZE as f32),
    )
}

fn update(ctx: &mut App, state: &mut State) {
    state.update(ctx);
}
