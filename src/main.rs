use rust_rl::*;
use notan::prelude::*;
use notan::math::{ vec2, vec3, Mat4, Vec2 };
use notan::draw::create_textures_from_atlas;
use notan::draw::{ CreateFont, CreateDraw, DrawImages, Draw, DrawTextSection, DrawShapes };
use specs::prelude::*;
use specs::saveload::{ SimpleMarker, SimpleMarkerAllocator };
use bracket_lib::prelude::*;
use std::collections::HashMap;
use crate::consts::{ DISPLAYHEIGHT, DISPLAYWIDTH, TILESIZE, FONTSIZE };
use crate::states::state::Fonts;

const WORK_SIZE: Vec2 = vec2(
    (DISPLAYWIDTH as f32) * TILESIZE.x,
    (DISPLAYHEIGHT as f32) * TILESIZE.x
);

#[notan_main]
fn main() -> Result<(), String> {
    let win_config = WindowConfig::new()
        .set_size(DISPLAYWIDTH * (TILESIZE.x as u32), DISPLAYHEIGHT * (TILESIZE.x as u32))
        .set_title("RUST-RL")
        .set_resizable(true)
        .set_fullscreen(true)
        .set_taskbar_icon_data(Some(include_bytes!("../resources/icon.png")))
        .set_vsync(true);
    notan
        ::init_with(setup)
        .add_config(win_config)
        .add_config(notan::draw::DrawConfig)
        .draw(draw)
        .update(update)
        .build()
}

fn setup(app: &mut App, gfx: &mut Graphics) -> State {
    effects::sound::init_sounds(app);
    effects::sound::ambience("a_relax");
    let texture = gfx
        .create_texture()
        .from_image(include_bytes!("../resources/atlas.png"))
        .build()
        .unwrap();
    let data = include_bytes!("../resources/atlas.json");
    let atlas = create_textures_from_atlas(data, &texture).unwrap();
    let texture = gfx
        .create_texture()
        .from_image(include_bytes!("../resources/td.png"))
        .build()
        .unwrap();
    let data = include_bytes!("../resources/td.json");
    let interface = create_textures_from_atlas(data, &texture).unwrap();
    let font = Fonts::new(
        gfx.create_font(include_bytes!("../resources/fonts/Greybeard-16px.ttf")).unwrap(),
        Some(
            gfx.create_font(include_bytes!("../resources/fonts/Greybeard-16px-Bold.ttf")).unwrap()
        ),
        Some(
            gfx.create_font(include_bytes!("../resources/fonts/Greybeard-16px-Italic.ttf")).unwrap()
        )
    );
    let mut gs = State {
        ecs: World::new(),
        //audio: sounds,
        atlas,
        interface,
        font,
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
    gs.ecs.register::<Avatar>();
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
    gs.ecs.register::<Stackable>();
    gs.ecs.register::<WantsToAssignKey>();
    gs.ecs.register::<Key>();
    gs.ecs.register::<WantsToRemoveKey>();
    gs.ecs.register::<WantsToDelete>();
    gs.ecs.register::<IntrinsicChanged>();
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
    gs.ecs.insert(RunState::MapGeneration {}); // TODO: Set this back to RunState::MapGen
    gs.ecs.insert(particle_system::ParticleBuilder::new());
    gs.ecs.insert(rex_assets::RexAssets::new());

    gamelog::setup_log();
    gamelog::record_event(consts::events::EVENT::Level(1));
    gs.generate_world_map(1, TileType::Floor);

    gs
}
const ASCII_MODE: bool = false; // Change this to config setting
const SHOW_BOUNDARIES: bool = false; // Config setting

#[derive(PartialEq)]
enum DrawType {
    None,
    Player,
    Visible,
    Telepathy,
}

#[derive(PartialEq, Eq, Hash)]
struct DrawKey {
    x: i32,
    y: i32,
    render_order: i32,
}
struct DrawInfo {
    e: Entity,
    draw_type: DrawType,
}

fn draw_entities(
    map: &Map,
    ecs: &World,
    draw: &mut Draw,
    atlas: &HashMap<String, Texture>,
    _font: &Fonts
) {
    {
        let bounds = crate::camera::get_screen_bounds(ecs, false);
        let bounds_to_px = bounds.to_px();
        let offset_x = bounds_to_px.x_offset - bounds_to_px.min_x;
        let offset_y = bounds_to_px.y_offset - bounds_to_px.min_y;
        let positions = ecs.read_storage::<Position>();
        let renderables = ecs.read_storage::<Renderable>();
        let hidden = ecs.read_storage::<Hidden>();
        let minds = ecs.read_storage::<Mind>();
        let pools = ecs.read_storage::<Pools>();
        let entities = ecs.entities();
        let player = ecs.read_storage::<Player>();
        let data = (&positions, &renderables, &entities, !&hidden).join().collect::<Vec<_>>();
        let mut to_draw: HashMap<DrawKey, DrawInfo> = HashMap::new();
        for (pos, render, e, _h) in data.iter() {
            let idx = map.xy_idx(pos.x, pos.y);
            if
                crate::camera::in_bounds(
                    pos.x,
                    pos.y,
                    bounds.min_x,
                    bounds.min_y,
                    bounds.max_x,
                    bounds.max_y
                )
            {
                let draw_type = if map.visible_tiles[idx] {
                    // If it's anything else, just draw it.
                    if player.get(*e).is_some() {
                        DrawType::Player
                    } else {
                        DrawType::Visible
                    }
                } else if map.telepath_tiles[idx] {
                    let has_mind = minds.get(*e);
                    if has_mind.is_some() {
                        // Mobs we see through telepathy - generally we just
                        // draw these, but it uses a unique enum variant so
                        // it can be treated differently if needed in future.
                        DrawType::Telepathy
                    } else {
                        DrawType::None
                    }
                } else {
                    // If we don't see it, and we don't sense it with
                    // telepathy, don't draw it at all.
                    DrawType::None
                };
                match draw_type {
                    DrawType::None => {}
                    _ => {
                        to_draw.insert(
                            DrawKey { x: pos.x, y: pos.y, render_order: render.render_order },
                            DrawInfo { e: *e, draw_type }
                        );
                    }
                }
            }
        }
        let mut entries: Vec<(&DrawKey, &DrawInfo)> = to_draw.iter().collect();
        entries.sort_by_key(|&(k, _v)| std::cmp::Reverse(k.render_order));
        for entry in entries.iter() {
            // TODO: ABSTRACT THESE INTO FUNCTIONS ONCE FUNCTIONALITY IS SETTLED ON.
            match entry.1.draw_type {
                DrawType::Visible | DrawType::Telepathy => {
                    let renderable = renderables.get(entry.1.e).unwrap();
                    let id = if let Some(sprite) = atlas.get(&renderable.sprite) {
                        sprite
                    } else {
                        panic!("No entity sprite found for ID: {}", &renderable.sprite);
                    };
                    let x_pos = (entry.0.x as f32) * TILESIZE.sprite_x + offset_x;
                    let y_pos = (entry.0.y as f32) * TILESIZE.sprite_y + offset_y;
                    let mul = themes::darken_by_distance(
                        Point::new(entry.0.x, entry.0.y),
                        *ecs.fetch::<Point>()
                    );
                    let col = Color::from_rgb(
                        renderable.fg.r * mul,
                        renderable.fg.g * mul,
                        renderable.fg.b * mul
                    );
                    draw.image(id)
                        .position(
                            x_pos + renderable.offset.0 * TILESIZE.sprite_x,
                            y_pos + renderable.offset.1 * TILESIZE.sprite_y
                        )
                        .color(col)
                        .size(TILESIZE.sprite_x, TILESIZE.sprite_y);
                    if let Some(pool) = pools.get(entry.1.e) {
                        if pool.hit_points.current < pool.hit_points.max {
                            draw_entity_hp(x_pos, y_pos, pool, draw);
                        }
                    }
                }
                DrawType::Player => {
                    let (x_pos, y_pos) = (
                        (entry.0.x as f32) * TILESIZE.sprite_x + offset_x,
                        (entry.0.y as f32) * TILESIZE.sprite_y + offset_y,
                    );
                    let textures = get_avatar_textures(ecs, atlas);
                    for (tex, col) in textures.iter() {
                        draw.image(tex)
                            .position(x_pos, y_pos)
                            .color(*col)
                            .size(TILESIZE.sprite_x, TILESIZE.sprite_y);
                    }
                }
                _ => {}
            }
        }
    }
}

fn get_avatar_textures(ecs: &World, atlas: &HashMap<String, Texture>) -> Vec<(Texture, Color)> {
    let player = ecs.fetch::<Entity>();
    let renderables = ecs.read_storage::<Renderable>();
    let equipped = ecs.read_storage::<Equipped>();
    let has_avatar = ecs.read_storage::<Avatar>();
    let mut avis = Vec::new();
    if let Some(renderables) = renderables.get(*player) {
        if let Some(sprite) = atlas.get(&renderables.sprite) {
            avis.push((
                sprite.clone(),
                Color::from_rgb(renderables.fg.r, renderables.fg.g, renderables.fg.b),
            ));
        } else {
            panic!("No player sprite found for ID: {}", &renderables.sprite);
        }
    } else {
        panic!("No player renderable found!");
    }
    for (_e, a, r) in (&equipped, &has_avatar, &renderables)
        .join()
        .filter(|item| item.0.owner == *player) {
        if let Some(sprite) = atlas.get(&a.sprite) {
            avis.push((sprite.clone(), Color::from_rgb(r.fg.r, r.fg.g, r.fg.b)));
        } else {
            panic!("No avatar sprite found for ID: {}", &a.sprite);
        }
    }
    avis
}

// Draws a HP bar LINE_WIDTH pixels thick centered above the entity.
fn draw_entity_hp(x: f32, y: f32, hp: &Pools, draw: &mut Draw) {
    const LINE_WIDTH: f32 = 3.0;
    let y = y + LINE_WIDTH + 1.0;
    let x = x;
    let fill_pct = (hp.hit_points.current as f32) / (hp.hit_points.max as f32);
    draw.line((x + 1.0, y), (x + (TILESIZE.sprite_x - 1.0) * fill_pct, y))
        .width(LINE_WIDTH)
        .color(Color::GREEN);
}

fn render_map_in_view(
    map: &Map,
    ecs: &World,
    draw: &mut Draw,
    atlas: &HashMap<String, Texture>,
    mapgen: bool
) {
    let bounds = crate::camera::get_screen_bounds(ecs, mapgen);
    let mut y = 0;
    for tile_y in bounds.min_y..bounds.max_y {
        let mut x = 0;
        for tile_x in bounds.min_x..bounds.max_x {
            if crate::camera::in_bounds(tile_x, tile_y, 0, 0, map.width, map.height) {
                let idx = map.xy_idx(tile_x, tile_y);
                if map.revealed_tiles[idx] || mapgen {
                    let draw_x =
                        (x as f32) * TILESIZE.sprite_x + (bounds.x_offset as f32) * TILESIZE.x;
                    let draw_y =
                        (y as f32) * TILESIZE.sprite_y + (bounds.y_offset as f32) * TILESIZE.x;
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
                        let sprite = if let Some(sprite) = atlas.get(id) {
                            sprite
                        } else {
                            panic!("No sprite found for ID: {}", id);
                        };
                        draw.image(sprite)
                            .position(draw_x, draw_y)
                            .color(tint)
                            .size(TILESIZE.sprite_x, TILESIZE.sprite_y);
                    }
                    if !map.visible_tiles[idx] {
                        // Recall map memory. TODO: Improve this? Optimize? Do we need to remember more fields?
                        if let Some(memories) = map.memory.get(&idx) {
                            let mut sorted: Vec<_> = memories.iter().collect();
                            sorted.sort_by(|a, b| a.render_order.cmp(&b.render_order));
                            for memory in sorted.iter() {
                                let mult = consts::visuals::NON_VISIBLE_MULTIPLIER;
                                let col = Color::from_rgb(
                                    memory.fg.r * mult,
                                    memory.fg.g * mult,
                                    memory.fg.b * mult
                                );
                                let sprite = if let Some(sprite) = atlas.get(&memory.sprite) {
                                    sprite
                                } else {
                                    panic!("No sprite found for ID: {}", memory.sprite);
                                };
                                draw.image(sprite)
                                    .position(
                                        draw_x + memory.offset.0 * TILESIZE.sprite_x,
                                        draw_y + memory.offset.1 * TILESIZE.sprite_y
                                    )
                                    .color(col)
                                    .size(TILESIZE.sprite_x, TILESIZE.sprite_y);
                            }
                        }
                    }
                }
            } else if SHOW_BOUNDARIES {
                // TODO: Draw boundaries
            }
            x += 1;
        }
        y += 1;
    }
}

struct BoxDraw {
    frame: String,
    fill: bool,
    top_left: (i32, i32),
    top_right: (i32, i32),
    bottom_left: (i32, i32),
    bottom_right: (i32, i32),
}
fn draw_spritebox(panel: BoxDraw, draw: &mut Draw, atlas: &HashMap<String, Texture>) {
    draw.image(atlas.get(&format!("{}_1", panel.frame)).unwrap()).position(
        (panel.top_left.0 as f32) * TILESIZE.x,
        (panel.top_left.1 as f32) * TILESIZE.x
    );
    for i in panel.top_left.0 + 1..panel.top_right.0 {
        draw.image(atlas.get(&format!("{}_2", panel.frame)).unwrap()).position(
            (i as f32) * TILESIZE.x,
            (panel.top_left.1 as f32) * TILESIZE.x
        );
    }
    draw.image(atlas.get(&format!("{}_3", panel.frame)).unwrap()).position(
        (panel.top_right.0 as f32) * TILESIZE.x,
        (panel.top_right.1 as f32) * TILESIZE.x
    );
    for i in panel.top_left.1 + 1..panel.bottom_left.1 {
        draw.image(atlas.get(&format!("{}_4", panel.frame)).unwrap()).position(
            (panel.top_left.0 as f32) * TILESIZE.x,
            (i as f32) * TILESIZE.x
        );
    }
    if panel.fill {
        for i in panel.top_left.0 + 1..panel.top_right.0 {
            for j in panel.top_left.1 + 1..panel.bottom_left.1 {
                draw.image(atlas.get(&format!("{}_5", panel.frame)).unwrap()).position(
                    (i as f32) * TILESIZE.x,
                    (j as f32) * TILESIZE.x
                );
            }
        }
    }
    for i in panel.top_right.1 + 1..panel.bottom_right.1 {
        draw.image(atlas.get(&format!("{}_6", panel.frame)).unwrap()).position(
            (panel.top_right.0 as f32) * TILESIZE.x,
            (i as f32) * TILESIZE.x
        );
    }
    draw.image(atlas.get(&format!("{}_7", panel.frame)).unwrap()).position(
        (panel.bottom_left.0 as f32) * TILESIZE.x,
        (panel.bottom_left.1 as f32) * TILESIZE.x
    );
    for i in panel.bottom_left.0 + 1..panel.bottom_right.0 {
        draw.image(atlas.get(&format!("{}_8", panel.frame)).unwrap()).position(
            (i as f32) * TILESIZE.x,
            (panel.bottom_left.1 as f32) * TILESIZE.x
        );
    }
    draw.image(atlas.get(&format!("{}_9", panel.frame)).unwrap()).position(
        (panel.bottom_right.0 as f32) * TILESIZE.x,
        (panel.bottom_right.1 as f32) * TILESIZE.x
    );
}

use crate::consts::visuals::{ VIEWPORT_H, VIEWPORT_W };
fn draw_bg(_ecs: &World, draw: &mut Draw, atlas: &HashMap<String, Texture>) {
    let offset = crate::camera::get_offset();
    let log = BoxDraw {
        frame: "line".to_string(),
        fill: false,
        top_left: (0, 0),
        top_right: (offset.x + VIEWPORT_W, 0),
        bottom_left: (0, offset.y - 2),
        bottom_right: (offset.x + VIEWPORT_W, offset.y - 2),
    };
    let game = BoxDraw {
        frame: "line".to_string(),
        fill: false,
        top_left: (offset.x - 1, offset.y - 1),
        top_right: (offset.x + VIEWPORT_W, offset.y - 1),
        bottom_left: (offset.x - 1, offset.y + VIEWPORT_H),
        bottom_right: (offset.x + VIEWPORT_W, offset.y + VIEWPORT_H),
    };
    let attr = BoxDraw {
        frame: "line".to_string(),
        fill: false,
        top_left: (offset.x - 1, offset.y + VIEWPORT_H + 1),
        top_right: (offset.x + VIEWPORT_W, offset.y + VIEWPORT_H + 1),
        bottom_left: (offset.x - 1, (DISPLAYHEIGHT as i32) - 1),
        bottom_right: (offset.x + VIEWPORT_W, (DISPLAYHEIGHT as i32) - 1),
    };
    let sidebox = BoxDraw {
        frame: "line".to_string(),
        fill: false,
        top_left: (offset.x + VIEWPORT_W + 1, 0),
        top_right: ((DISPLAYWIDTH as i32) - 1, 0),
        bottom_left: (offset.x + VIEWPORT_W + 1, (DISPLAYHEIGHT as i32) - 1),
        bottom_right: ((DISPLAYWIDTH as i32) - 1, (DISPLAYHEIGHT as i32) - 1),
    };
    draw_spritebox(log, draw, atlas);
    draw_spritebox(game, draw, atlas);
    draw_spritebox(attr, draw, atlas);
    draw_spritebox(sidebox, draw, atlas);
}

fn draw(_app: &mut App, gfx: &mut Graphics, gs: &mut State) {
    let mut draw = gfx.create_draw();
    draw.clear(Color::BLACK);
    let mut log = false;
    let runstate = *gs.ecs.fetch::<RunState>();
    match runstate {
        RunState::MainMenu { .. } => {
            gui::draw_mainmenu(&gs.ecs, &mut draw, &gs.atlas, &gs.font);
        }
        RunState::CharacterCreation { .. } => {
            gui::draw_charcreation(&gs.ecs, &mut draw, &gs.atlas, &gs.font);
        }
        RunState::PreRun { .. } => {}
        RunState::MapGeneration => {
            draw_bg(&gs.ecs, &mut draw, &gs.interface);
            if config::CONFIG.logging.show_mapgen && gs.mapgen_history.len() > 0 {
                render_map_in_view(
                    &gs.mapgen_history[gs.mapgen_index],
                    &gs.ecs,
                    &mut draw,
                    &gs.atlas,
                    true
                );
            }
            gui::draw_ui2(&gs.ecs, &mut draw, &gs.atlas, &gs.font);
        }
        _ => {
            let map = gs.ecs.fetch::<Map>();
            draw_bg(&gs.ecs, &mut draw, &gs.interface);
            render_map_in_view(&*map, &gs.ecs, &mut draw, &gs.atlas, false);
            // Special case: targeting needs to be drawn *below* entities, but above tiles.
            if let RunState::ShowTargeting { range, item: _, x, y, aoe } = runstate {
                gui::draw_targeting(&gs.ecs, &mut draw, &gs.atlas, x, y, range, aoe);
            }
            draw_entities(&*map, &gs.ecs, &mut draw, &gs.atlas, &gs.font);
            gui::draw_ui2(&gs.ecs, &mut draw, &gs.atlas, &gs.font);
            log = true;
        }
    }
    match runstate {
        RunState::Farlook { x, y } => {
            gui::draw_farlook(&gs.ecs, x, y, &mut draw, &gs.atlas);
            //draw_tooltips(&gs.ecs, ctx, Some((x, y))); TODO: Put this in draw loop
        }
        RunState::ShowCheatMenu => {
            gui::draw_cheat_menu(&mut draw, &gs.atlas, &gs.font);
        }
        RunState::ActionWithDirection { .. } => {
            corner_text("In what direction? [0-9]/[YUHJKLBN]", &mut draw, &gs.font);
        }
        RunState::GameOver => {
            corner_text("Create morgue file? [Y/N]", &mut draw, &gs.font);
        }
        RunState::ShowInventory => {
            corner_text("Use what? [aA-zZ]/[Esc.]", &mut draw, &gs.font);
            let offset = crate::camera::get_offset();
            let (x, y) = (
                ((1 + offset.x) as f32) * TILESIZE.x,
                ((3 + offset.y) as f32) * TILESIZE.x,
            );
            gui::draw_backpack_items(&gs.ecs, &mut draw, &gs.font, x, y);
        }
        RunState::ShowDropItem => {
            corner_text("Drop what? [aA-zZ]/[Esc.]", &mut draw, &gs.font);
            let offset = crate::camera::get_offset();
            let (x, y) = (
                ((1 + offset.x) as f32) * TILESIZE.x,
                ((3 + offset.y) as f32) * TILESIZE.x,
            );
            gui::draw_backpack_items(&gs.ecs, &mut draw, &gs.font, x, y);
        }
        RunState::ShowRemoveItem => {
            corner_text("Unequip which item? [aA-zZ]/[Esc.]", &mut draw, &gs.font);
            let offset = crate::camera::get_offset();
            let (x, y) = (
                ((1 + offset.x) as f32) * TILESIZE.x,
                ((3 + offset.y) as f32) * TILESIZE.x,
            );
            gui::draw_items(&gs.ecs, &mut draw, &gs.font, x, y, gui::Location::Equipped, None);
        }
        RunState::ShowTargeting { .. } => {
            corner_text("Targeting which tile? [0-9]/[YUHJKLBN]", &mut draw, &gs.font);
        }
        RunState::HelpScreen => {
            corner_text("The help screen is a placeholder! [?]", &mut draw, &gs.font);
        }
        _ => {}
    }
    // TODO: Once the rest of drawing is finalised, this should be abstracted
    // into some functions that make it easier to tell what is going on. But
    // for the short-term:
    // 1. notan::Text is required for rich text drawing, rather than just the
    //    basics that are accessible with notan::Draw's .text() method.
    // 2. notan::Text cannot be projected, and rendering both Draw and Text
    //    requires two GPU calls instead of just one.
    // 3. To fix this, our log is drawn to notan::Text, then rendered to a
    //    render texture, and applied as any other image to notan::Draw.
    // 4. notan::Draw is projected, and then rendered, and everything works.
    // Further stuff: Make the render texture only as large as is required,
    //                so text cannot escape the bounds of the logbox.
    let (width, height) = gfx.size();
    let win_size = vec2(width as f32, height as f32);
    let (projection, _) = calc_projection(win_size, WORK_SIZE);
    if log {
        let buffer = gfx
            .create_render_texture(width, height)
            .build()
            .expect("Failed to create render texture");
        gamelog::render_log(
            &buffer,
            gfx,
            &gs.font,
            &(TILESIZE.x, TILESIZE.x * 6.0 + 4.0),
            (VIEWPORT_W as f32) * TILESIZE.x,
            5
        );
        draw.image(&buffer)
            .position(0.0, 0.0)
            .size(width as f32, height as f32);
    }
    draw.set_projection(Some(projection));
    gfx.render(&draw);
}

fn update(ctx: &mut App, state: &mut State) {
    state.update(ctx);
}

fn corner_text(text: &str, draw: &mut Draw, font: &Fonts) {
    let offset = crate::camera::get_offset();
    draw.text(&font.b(), &text)
        .position(((offset.x + 1) as f32) * TILESIZE.x, ((offset.y + 1) as f32) * TILESIZE.x)
        .size(FONTSIZE);
}

fn calc_projection(win_size: Vec2, work_size: Vec2) -> (Mat4, f32) {
    let ratio = (win_size.x / work_size.x).min(win_size.y / work_size.y);
    let proj = Mat4::orthographic_rh_gl(0.0, win_size.x, win_size.y, 0.0, -1.0, 1.0);
    let scale = Mat4::from_scale(vec3(ratio, ratio, 1.0));
    let position = vec3(
        (win_size.x - work_size.x * ratio) * 0.5,
        (win_size.y - work_size.y * ratio) * 0.5,
        1.0
    );
    let trans = Mat4::from_translation(position);
    (proj * trans * scale, ratio)
}
