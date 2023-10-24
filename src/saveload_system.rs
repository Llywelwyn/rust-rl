use super::components::*;
use bracket_lib::prelude::*;
use specs::prelude::*;
use specs::saveload::{
    DeserializeComponents,
    MarkedBuilder,
    SerializeComponents,
    SimpleMarker,
    SimpleMarkerAllocator,
};
use std::fs;
use std::fs::File;
use std::path::Path;
use std::convert::Infallible;

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $($type:ty),*) => {
        $(
        SerializeComponents::<Infallible, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

#[cfg(target_arch = "wasm32")]
pub fn save_game(_ecs: &mut World) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(ecs: &mut World) {
    // Create helper
    let mapcopy = ecs.get_mut::<super::map::Map>().unwrap().clone();
    let dungeon_master = ecs.get_mut::<super::map::MasterDungeonMap>().unwrap().clone();
    let savehelper = ecs
        .create_entity()
        .with(SerializationHelper { map: mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    let savehelper2 = ecs
        .create_entity()
        .with(DMSerializationHelper {
            map: dungeon_master,
            log: crate::gamelog::clone_log(),
            event_counts: crate::gamelog::clone_event_counts(),
            events: crate::gamelog::clone_events(),
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Actually serialize
    {
        let data = (ecs.entities(), ecs.read_storage::<SimpleMarker<SerializeMe>>());

        let writer = File::create("./savegame.bin").unwrap();
        let mut serializer = bincode::Serializer::new(writer, bincode::options());
        serialize_individually!(
            ecs,
            serializer,
            data,
            AOE,
            ArmourClassBonus,
            Attributes,
            Beatitude,
            Bleeds,
            Blind,
            BlocksTile,
            BlocksVisibility,
            Burden,
            Chasing,
            Clock,
            Confusion,
            Consumable,
            Destructible,
            Digger,
            Door,
            Energy,
            EntityMoved,
            EntryTrigger,
            EquipmentChanged,
            Equippable,
            Equipped,
            Faction,
            GrantsSpell,
            GrantsXP,
            HasAncestry,
            HasClass,
            HasDamageModifiers,
            Hidden,
            HungerClock,
            IdentifiedBeatitude,
            IdentifiedItem,
            InBackpack,
            InflictsDamage,
            IntrinsicChanged,
            Intrinsics,
            Item,
            KnownSpells,
            LootTable,
            MagicItem,
            MagicMapper,
            MeleeWeapon,
            Mind,
            MoveMode,
            MultiAttack,
            NaturalAttacks,
            Name,
            ObfuscatedName,
            OtherLevelPosition,
            ParticleLifetime,
            Player,
            Pools,
            Position,
            Prop,
            ProvidesHealing,
            ProvidesIdentify,
            ProvidesNutrition,
            ProvidesRemoveCurse,
            Quips,
            Ranged,
            Renderable,
            SingleActivation,
            Skills,
            SpawnParticleBurst,
            SpawnParticleLine,
            SpawnParticleSimple,
            TakingTurn,
            Telepath,
            ToHitBonus,
            Viewshed,
            Charges,
            WantsToApproach,
            WantsToDropItem,
            WantsToFlee,
            WantsToMelee,
            WantsToPickupItem,
            WantsToRemoveItem,
            WantsToUseItem,
            SerializationHelper,
            DMSerializationHelper
        );
    }

    // Clean up
    ecs.delete_entity(savehelper).expect("<savehelper> Crash on cleanup");
    ecs.delete_entity(savehelper2).expect("<savehelper2> Crash on cleanup");
}

pub fn does_save_exist() -> bool {
    Path::new("./savegame.bin").exists()
}

macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $($type:ty),*) => {
        $(
        DeserializeComponents::<Infallible, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &$data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocater
            &mut $de,
        )
        .unwrap();
        )*
    };
}

pub fn load_game(ecs: &mut World) {
    {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            ecs.delete_entity(*del).expect("Deletion failed");
        }
    }

    let data = fs::read("./savegame.bin").unwrap();
    let mut de = bincode::Deserializer::with_reader(&*data, bincode::options());

    {
        let mut d = (
            &mut ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>(),
        );

        deserialize_individually!(
            ecs,
            de,
            d,
            AOE,
            ArmourClassBonus,
            Attributes,
            Beatitude,
            Bleeds,
            Blind,
            BlocksTile,
            BlocksVisibility,
            Burden,
            Chasing,
            Clock,
            Confusion,
            Consumable,
            Destructible,
            Digger,
            Door,
            Energy,
            EntityMoved,
            EntryTrigger,
            EquipmentChanged,
            Equippable,
            Equipped,
            Faction,
            GrantsSpell,
            GrantsXP,
            HasAncestry,
            HasClass,
            HasDamageModifiers,
            Hidden,
            HungerClock,
            IdentifiedBeatitude,
            IdentifiedItem,
            InBackpack,
            InflictsDamage,
            IntrinsicChanged,
            Intrinsics,
            Item,
            KnownSpells,
            LootTable,
            MagicItem,
            MagicMapper,
            MeleeWeapon,
            Mind,
            MoveMode,
            MultiAttack,
            NaturalAttacks,
            Name,
            ObfuscatedName,
            OtherLevelPosition,
            ParticleLifetime,
            Player,
            Pools,
            Position,
            Prop,
            ProvidesHealing,
            ProvidesIdentify,
            ProvidesNutrition,
            ProvidesRemoveCurse,
            Quips,
            Ranged,
            Renderable,
            SingleActivation,
            Skills,
            SpawnParticleBurst,
            SpawnParticleLine,
            SpawnParticleSimple,
            TakingTurn,
            Telepath,
            ToHitBonus,
            Viewshed,
            Charges,
            WantsToApproach,
            WantsToDropItem,
            WantsToFlee,
            WantsToMelee,
            WantsToPickupItem,
            WantsToRemoveItem,
            WantsToUseItem,
            SerializationHelper,
            DMSerializationHelper
        );
    }

    let mut deleteme: Option<Entity> = None;
    let mut deleteme2: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let helper2 = ecs.read_storage::<DMSerializationHelper>();
        let player = ecs.read_storage::<Player>();
        let position = ecs.read_storage::<Position>();
        for (e, h) in (&entities, &helper).join() {
            let mut worldmap = ecs.write_resource::<super::map::Map>();
            *worldmap = h.map.clone();
            crate::spatial::set_size((worldmap.width * worldmap.height) as usize);
            deleteme = Some(e);
        }
        for (e, h) in (&entities, &helper2).join() {
            let mut dungeonmaster = ecs.write_resource::<super::map::MasterDungeonMap>();
            *dungeonmaster = h.map.clone();
            deleteme2 = Some(e);
            crate::gamelog::restore_log(&mut h.log.clone());
            crate::gamelog::restore_event_counter(h.event_counts.clone());
            crate::gamelog::restore_events(h.events.clone());
        }
        for (e, _p, pos) in (&entities, &player, &position).join() {
            let mut ppos = ecs.write_resource::<Point>();
            *ppos = Point::new(pos.x, pos.y);
            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = e;
        }
    }
    ecs.delete_entity(deleteme.unwrap()).expect("<deleteme> Unable to delete helper");
    ecs.delete_entity(deleteme2.unwrap()).expect("<deleteme2> Unable to delete helper");
}

pub fn delete_save() {
    if Path::new("./savegame.bin").exists() {
        std::fs::remove_file("./savegame.bin").expect("Unable to delete file");
    }
}
