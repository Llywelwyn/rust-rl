use rust_rl::*;
use notan::prelude::*;
use notan::draw::create_textures_from_atlas;
use notan::draw::{ CreateFont, CreateDraw, DrawImages, Draw, DrawTextSection, DrawShapes };
use specs::prelude::*;
use specs::saveload::{ SimpleMarker, SimpleMarkerAllocator };
use bracket_lib::prelude::*;
use std::collections::HashMap;
use crate::consts::{ DISPLAYHEIGHT, DISPLAYWIDTH, TILESIZE, FONTSIZE };
use crate::states::state::Fonts;

#[notan_main]
fn main() -> Result<(), String> {
    let win_config = WindowConfig::new()
        .set_size(DISPLAYWIDTH * (TILESIZE as u32), DISPLAYHEIGHT * (TILESIZE as u32))
        .set_title("RUST-RL")
        .set_resizable(false)
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

fn setup(gfx: &mut Graphics) -> State {
    let texture = gfx
        .create_texture()
        .from_image(include_bytes!("../resources/td.png"))
        .build()
        .unwrap();
    let data = include_bytes!("../resources/td.json");
    let atlas = create_textures_from_atlas(data, &texture).unwrap();
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
        base_texture: texture,
        atlas,
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
enum DrawType {
    None,
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

fn draw_camera(
    map: &Map,
    ecs: &World,
    draw: &mut Draw,
    atlas: &HashMap<String, Texture>,
    font: &Fonts
) {
    render_map_in_view(&*map, ecs, draw, atlas, false);
    {
        let bounds = crate::camera::get_screen_bounds(ecs, false);
        let positions = ecs.read_storage::<Position>();
        let renderables = ecs.read_storage::<Renderable>();
        let hidden = ecs.read_storage::<Hidden>();
        let minds = ecs.read_storage::<Mind>();
        let pools = ecs.read_storage::<Pools>();
        let entities = ecs.entities();
        let data = (&positions, &renderables, &entities, !&hidden).join().collect::<Vec<_>>();
        let mut to_draw: HashMap<DrawKey, DrawInfo> = HashMap::new();
        for (pos, render, e, _h) in data.iter() {
            let idx = map.xy_idx(pos.x, pos.y);
            let offset_x = pos.x - bounds.min_x + bounds.x_offset;
            let offset_y = pos.y - bounds.min_y + bounds.y_offset;
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
                    DrawType::Visible
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
                            DrawKey { x: offset_x, y: offset_y, render_order: render.render_order },
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
                    if let Some(spriteinfo) = &renderable.sprite {
                        let id = if let Some(sprite) = atlas.get(&spriteinfo.id) {
                            sprite
                        } else {
                            panic!("No entity sprite found for ID: {}", spriteinfo.id);
                        };
                        draw.image(id)
                            .position(
                                ((entry.0.x as f32) + spriteinfo.offset.0) * TILESIZE,
                                ((entry.0.y as f32) + spriteinfo.offset.1) * TILESIZE
                            )
                            .color(
                                if spriteinfo.recolour {
                                    Color::from_rgb(
                                        renderable.fg.r,
                                        renderable.fg.g,
                                        renderable.fg.b
                                    )
                                } else {
                                    let mul = themes::darken_by_distance(
                                        Point::new(
                                            entry.0.x + bounds.min_x - bounds.x_offset,
                                            entry.0.y + bounds.min_y - bounds.y_offset
                                        ),
                                        *ecs.fetch::<Point>()
                                    );
                                    Color::from_rgb(mul, mul, mul)
                                }
                            );
                        if let Some(pool) = pools.get(entry.1.e) {
                            if pool.hit_points.current < pool.hit_points.max {
                                gui::draw_bar(
                                    draw,
                                    entry.0.x as f32,
                                    entry.0.y as f32,
                                    1.0,
                                    1.0,
                                    pool.hit_points.current,
                                    pool.hit_points.max,
                                    Color::GREEN,
                                    Color::RED
                                );
                            }
                        }
                    } else {
                        // Fallback to drawing text.
                        draw.text(
                            &font.b(),
                            &format!("{}", bracket_lib::terminal::to_char(renderable.glyph as u8))
                        )
                            .position(
                                ((entry.0.x as f32) + 0.5) * TILESIZE,
                                ((entry.0.y as f32) + 0.5) * TILESIZE
                            )
                            .color(
                                Color::from_rgb(renderable.fg.r, renderable.fg.g, renderable.fg.b)
                            )
                            .size(FONTSIZE)
                            .h_align_center()
                            .v_align_middle();
                    }
                }
                _ => {}
            }
        }
    }
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
                            .position(
                                ((x + bounds.x_offset) as f32) * TILESIZE,
                                ((y + bounds.y_offset) as f32) * TILESIZE
                            )
                            .color(tint);
                    }
                    if !map.visible_tiles[idx] {
                        // Recall map memory. TODO: Improve this? Optimize? Do we need to remember more fields?
                        if let Some(memories) = map.memory.get(&idx) {
                            let mut sorted: Vec<_> = memories.iter().collect();
                            sorted.sort_by(|a, b| a.render_order.cmp(&b.render_order));
                            for memory in sorted.iter() {
                                let sprite = if let Some(sprite) = atlas.get(&memory.sprite) {
                                    sprite
                                } else {
                                    panic!("No sprite found for ID: {}", memory.sprite);
                                };
                                draw.image(sprite)
                                    .position(
                                        (((x + bounds.x_offset) as f32) + memory.offset.0) *
                                            TILESIZE,
                                        (((y + bounds.y_offset) as f32) + memory.offset.1) *
                                            TILESIZE
                                    )
                                    .color(
                                        if memory.recolour {
                                            Color::from_rgb(memory.fg.r, memory.fg.g, memory.fg.b)
                                        } else {
                                            let mult = 0.3;
                                            Color::from_rgb(mult, mult, mult)
                                        }
                                    );
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
        (panel.top_left.0 as f32) * TILESIZE,
        (panel.top_left.1 as f32) * TILESIZE
    );
    for i in panel.top_left.0 + 1..panel.top_right.0 {
        draw.image(atlas.get(&format!("{}_2", panel.frame)).unwrap()).position(
            (i as f32) * TILESIZE,
            (panel.top_left.1 as f32) * TILESIZE
        );
    }
    draw.image(atlas.get(&format!("{}_3", panel.frame)).unwrap()).position(
        (panel.top_right.0 as f32) * TILESIZE,
        (panel.top_right.1 as f32) * TILESIZE
    );
    for i in panel.top_left.1 + 1..panel.bottom_left.1 {
        draw.image(atlas.get(&format!("{}_4", panel.frame)).unwrap()).position(
            (panel.top_left.0 as f32) * TILESIZE,
            (i as f32) * TILESIZE
        );
    }
    if panel.fill {
        for i in panel.top_left.0 + 1..panel.top_right.0 {
            for j in panel.top_left.1 + 1..panel.bottom_left.1 {
                draw.image(atlas.get(&format!("{}_5", panel.frame)).unwrap()).position(
                    (i as f32) * TILESIZE,
                    (j as f32) * TILESIZE
                );
            }
        }
    }
    for i in panel.top_right.1 + 1..panel.bottom_right.1 {
        draw.image(atlas.get(&format!("{}_6", panel.frame)).unwrap()).position(
            (panel.top_right.0 as f32) * TILESIZE,
            (i as f32) * TILESIZE
        );
    }
    draw.image(atlas.get(&format!("{}_7", panel.frame)).unwrap()).position(
        (panel.bottom_left.0 as f32) * TILESIZE,
        (panel.bottom_left.1 as f32) * TILESIZE
    );
    for i in panel.bottom_left.0 + 1..panel.bottom_right.0 {
        draw.image(atlas.get(&format!("{}_8", panel.frame)).unwrap()).position(
            (i as f32) * TILESIZE,
            (panel.bottom_left.1 as f32) * TILESIZE
        );
    }
    draw.image(atlas.get(&format!("{}_9", panel.frame)).unwrap()).position(
        (panel.bottom_right.0 as f32) * TILESIZE,
        (panel.bottom_right.1 as f32) * TILESIZE
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
            draw_bg(&gs.ecs, &mut draw, &gs.atlas);
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
            draw_bg(&gs.ecs, &mut draw, &gs.atlas);
            draw_camera(&*map, &gs.ecs, &mut draw, &gs.atlas, &gs.font);
            gui::draw_ui2(&gs.ecs, &mut draw, &gs.atlas, &gs.font);
            log = true;
        }
    }
    match runstate {
        RunState::Farlook { x, y } => {
            gui::draw_farlook(x, y, &mut draw, &gs.atlas);
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
            gui::draw_inventory(&gs.ecs, &mut draw, &gs.font, 1, 3);
        }
        RunState::ShowDropItem => {
            corner_text("Drop what? [aA-zZ]/[Esc.]", &mut draw, &gs.font);
            gui::draw_inventory(&gs.ecs, &mut draw, &gs.font, 1, 3);
        }
        _ => {}
    }
    gfx.render(&draw);
    gamelog::render(log, gfx, &gs.font);
}

fn update(ctx: &mut App, state: &mut State) {
    state.update(ctx);
}

fn corner_text(text: &str, draw: &mut Draw, font: &Fonts) {
    let offset = crate::camera::get_offset();
    draw.text(&font.b(), &text)
        .position(((offset.x + 1) as f32) * TILESIZE, ((offset.y + 1) as f32) * TILESIZE)
        .size(FONTSIZE);
}
