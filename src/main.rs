use rust_rl::*;
use specs::prelude::*;
use specs::saveload::{ SimpleMarker, SimpleMarkerAllocator };
use rltk::prelude::*;

const DISPLAYWIDTH: i32 = 72;
const DISPLAYHEIGHT: i32 = 40;

fn main() -> rltk::BError {
    // Embedded resources for use in wasm build
    const MAIN_22_20_BYTES: &[u8] = include_bytes!("../resources/nagidal22x20_centred.png");
    const TEXT_11_20_BYTES: &[u8] = include_bytes!("../resources/curses11x20.png");
    const SINGLE_1_1_BYTES: &[u8] = include_bytes!("../resources/healthbar22x2.png");
    rltk::embedding::EMBED
        .lock()
        .add_resource("resources/nagidal22x20_centred.png".to_string(), MAIN_22_20_BYTES);
    rltk::embedding::EMBED
        .lock()
        .add_resource("resources/curses11x20.png".to_string(), TEXT_11_20_BYTES);
    rltk::embedding::EMBED
        .lock()
        .add_resource("resources/healthbar22x2.png".to_string(), SINGLE_1_1_BYTES);

    let mut context = RltkBuilder::new()
        .with_title("rust-rl")
        .with_dimensions(DISPLAYWIDTH, DISPLAYHEIGHT)
        .with_font("nagidal22x20_centred.png", 22, 20)
        .with_font("curses11x20.png", 11, 20)
        .with_font("healthbar22x2.png", 1, 1)
        .with_tile_dimensions(22, 20)
        .with_simple_console(DISPLAYWIDTH, DISPLAYHEIGHT, "nagidal22x20_centred.png")
        .with_sparse_console(DISPLAYWIDTH * 2, DISPLAYHEIGHT, "curses11x20.png")
        .with_sparse_console(DISPLAYWIDTH * 22, DISPLAYHEIGHT * 20, "healthbar22x2.png")
        .build()?;
    if config::CONFIG.visuals.with_scanlines {
        context.with_post_scanlines(config::CONFIG.visuals.with_screen_burn);
    }

    let mut gs = State {
        ecs: World::new(),
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
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<DMSerializationHelper>();
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    raws::load_raws();

    // Insert calls
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    gs.ecs.insert(map::MasterDungeonMap::new()); // Master map list
    gs.ecs.insert(Map::new(true, 1, 64, 64, 0, "New Map", "N", 0)); // Map
    gs.ecs.insert(Point::new(0, 0)); // Player pos
    gs.ecs.insert(gui::Ancestry::Dwarf); // ancestry
    let player_entity = spawner::player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(player_entity); // Player entity
    gs.ecs.insert(RunState::MapGeneration {}); // RunState
    gs.ecs.insert(particle_system::ParticleBuilder::new());
    gs.ecs.insert(rex_assets::RexAssets::new());

    gamelog::setup_log();
    gamelog::record_event(data::events::EVENT::LEVEL(1));
    gs.generate_world_map(1, TileType::Floor);

    rltk::main_loop(context, gs)
}
