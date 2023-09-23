use std::fs::{ File, create_dir_all };
use std::io::{ self, Write };
use std::time::SystemTime;
use super::Map;
use crate::gamelog;
use crate::components::*;
use crate::gui::{ Class, Ancestry, unobf_name_ecs };
use specs::prelude::*;
use bracket_lib::prelude::*;
use to_char;
use std::collections::HashMap;
use crate::data::events::*;

#[cfg(target_arch = "wasm32")]
pub fn create_morgue_file(ecs: &World) {
    console::log("wasm32 doesn't support writing files yet, so here's the morgue info:");
    let morgue_info = create_morgue_string(ecs);
    console::log(morgue_info);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_morgue_file(ecs: &World) {
    let morgue_dir = "morgue";
    if let Err(err) = create_dir_all(&morgue_dir) {
        console::log(format!("Unable to create the directory (/{}): {}", morgue_dir, err));
    }
    let morgue_info = create_morgue_string(ecs);
    let file_name = create_file_name(ecs, morgue_dir);
    if let Err(err) = write_morgue_file(file_name.as_str(), morgue_info.as_str()) {
        console::log(format!("Unable to write the morgue file: {}", err));
    };
}

fn create_file_name(ecs: &World, morgue_dir: &str) -> String {
    let e = ecs.fetch::<Entity>();
    let pools = ecs.read_storage::<Pools>();
    let pool = pools.get(*e).unwrap();
    let class = match ecs.read_storage::<HasClass>().get(*e).unwrap().name {
        Class::Fighter => "fighter",
        Class::Wizard => "wizard",
        Class::Rogue => "rogue",
        Class::Villager => "villager",
    };
    let ancestry = match ecs.read_storage::<HasAncestry>().get(*e).unwrap().name {
        Ancestry::Human => "human",
        Ancestry::Elf => "elf",
        Ancestry::Dwarf => "dwarf",
        Ancestry::Gnome => "gnome",
        Ancestry::Catfolk => "catfolk",
        Ancestry::NULL => "NULL",
    };
    return format!(
        "{}/lv{}-{}-{}-{}.txt",
        morgue_dir,
        &pool.level,
        &ancestry,
        &class,
        get_timestamp()
    );
}

fn create_morgue_string(ecs: &World) -> String {
    // Initialise default
    let mut morgue_info: String = Default::default();
    let e = ecs.fetch::<Entity>();
    let class = match ecs.read_storage::<HasClass>().get(*e).unwrap().name {
        Class::Fighter => "fighter",
        Class::Wizard => "wizard",
        Class::Rogue => "rogue",
        Class::Villager => "villager",
    };
    let ancestry = match ecs.read_storage::<HasAncestry>().get(*e).unwrap().name {
        Ancestry::Human => "human",
        Ancestry::Elf => "elf",
        Ancestry::Dwarf => "dwarf",
        Ancestry::Gnome => "gnome",
        Ancestry::Catfolk => "catfolk",
        Ancestry::NULL => "NULL",
    };
    let pools = ecs.read_storage::<Pools>();
    let pool = pools.get(*e).unwrap();
    let header = format!("{} {}, level {}/{}", &ancestry, &class, &pool.level, &pool.xp);
    morgue_info.push_str(&create_boxed_text(header.as_str(), None));
    morgue_info.push_str(&draw_tombstone(ecs, header.len()));
    morgue_info.push_str(&draw_map(ecs));
    morgue_info.push_str("\n");
    morgue_info.push_str(&create_boxed_text("Equipment", None));
    morgue_info.push_str(&draw_equipment(ecs));
    morgue_info.push_str(&create_boxed_text("Backpack", None));
    morgue_info.push_str(&draw_backpack(ecs));
    morgue_info.push_str(&create_boxed_text("Significant Events", None));
    morgue_info.push_str(&draw_events_list());

    return morgue_info;
}

fn write_morgue_file(file_name: &str, morgue_info: &str) -> Result<(), io::Error> {
    // Save to file
    let mut file = File::create(&file_name)?; // Open/create morgue file
    file.write_all(morgue_info.as_bytes())?;
    Ok(())
}

fn get_timestamp() -> String {
    return SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string();
}

fn create_boxed_text(content: &str, width: Option<usize>) -> String {
    let width = if width.is_some() { width.unwrap() } else { content.len() + 2 };
    let horizontal = format!("{:═^w$}", "", w = width);
    return format!("╔{h}╗\n║ {c} ║\n╚{h}╝\n", h = horizontal, c = content);
}

fn draw_tombstone(ecs: &World, len: usize) -> String {
    let pad = (len - 17) / 2;
    let map = ecs.fetch::<Map>();
    let pools = ecs.read_storage::<Pools>();
    let pool = pools.get(*ecs.fetch::<Entity>()).unwrap();
    let attrs = ecs.read_storage::<Attributes>();
    let attr = attrs.get(*ecs.fetch::<Entity>()).unwrap();
    return format!(
        "{:^p$}    .-'~~~`-.      HP {}/{}    MP {}/{}\n{:^p$}  .'         `.\n{:^p$}  |  rest     |    STR {:>2} ({:+})  CON {:>2} ({:+})  WIS {:>2} ({:+})\n{:^p$}  |    in     |    DEX {:>2} ({:+})  INT {:>2} ({:+})  CHA {:>2} ({:+})\n{:^p$}  |     peace |\n{:^p$}\\\\|           |//  You died in {} [id {}], after {} turns.\n{:^p$}^^^^^^^^^^^^^^^^^{:^p$}\n",
        "",
        pool.hit_points.current,
        pool.hit_points.max,
        pool.mana.current,
        pool.mana.max,
        "",
        "",
        attr.strength.base + attr.strength.modifiers,
        attr.strength.bonus,
        attr.constitution.base + attr.constitution.modifiers,
        attr.constitution.bonus,
        attr.wisdom.base + attr.wisdom.modifiers,
        attr.wisdom.bonus,
        "",
        attr.dexterity.base + attr.dexterity.modifiers,
        attr.dexterity.bonus,
        attr.intelligence.base + attr.intelligence.modifiers,
        attr.intelligence.bonus,
        attr.charisma.base + attr.charisma.modifiers,
        attr.charisma.bonus,
        "",
        "",
        map.name,
        map.id,
        gamelog::get_event_count(EVENT::COUNT_TURN),
        "",
        "",
        p = pad
    );
}

fn draw_map(ecs: &World) -> String {
    let map = ecs.fetch::<Map>();
    let mut result: String = Default::default();
    let point = ecs.fetch::<Point>();
    for y in 0..map.height {
        for x in 0..map.width {
            let idx = map.xy_idx(x, y);
            let mut glyph_u16: u16 = 0;
            if idx == map.xy_idx(point.x, point.y) {
                glyph_u16 = to_cp437('@');
            } else if crate::spatial::has_tile_content(idx) {
                let mut render_order = 0;
                crate::spatial::for_each_tile_content(idx, |e| {
                    if let Some(renderable) = ecs.read_storage::<Renderable>().get(e) {
                        if renderable.render_order >= render_order {
                            render_order = renderable.render_order;
                            glyph_u16 = renderable.glyph;
                        }
                    }
                });
            } else {
                glyph_u16 = crate::map::themes::get_tile_renderables_for_id(
                    idx,
                    &*map,
                    None,
                    Some(true)
                ).0;
            }
            let char = to_char((glyph_u16 & 0xff) as u8);
            result.push_str(&char.to_string());
        }
        result.push_str("\n");
    }
    return result;
}

fn draw_equipment(ecs: &World) -> String {
    // Get all of the player's equipment.
    let mut equip: HashMap<Entity, i32> = HashMap::new();
    let equipped = ecs.read_storage::<Equipped>();
    for (entity, _e, _n) in (&ecs.entities(), &equipped, &ecs.read_storage::<Name>())
        .join()
        .filter(|item| item.1.owner == *ecs.fetch::<Entity>()) {
        equip
            .entry(entity)
            .and_modify(|count| {
                *count += 1;
            })
            .or_insert(1);
    }
    let mut result: String = Default::default();
    for item in equip {
        let slot = match equipped.get(item.0).unwrap().slot {
            EquipmentSlot::Melee => "l-hand -",
            EquipmentSlot::Shield => "r-hand -",
            EquipmentSlot::Head => "head -",
            EquipmentSlot::Body => "body -",
            EquipmentSlot::Feet => "feet -",
            EquipmentSlot::Hands => "hands -",
            EquipmentSlot::Back => "back -",
            EquipmentSlot::Neck => "neck -",
        };
        let name = if item.1 != 1 {
            unobf_name_ecs(ecs, item.0).1
        } else {
            unobf_name_ecs(ecs, item.0).0
        };
        result.push_str(&format!("{:>8} {}\n", slot, name));
    }
    result.push_str("\n");
    return result;
}

fn draw_backpack(ecs: &World) -> String {
    // Get all of the player's backpack.
    let mut pack: HashMap<(String, String), (i32, Entity)> = HashMap::new();
    for (entity, _bp, _n) in (
        &ecs.entities(),
        &ecs.read_storage::<InBackpack>(),
        &ecs.read_storage::<Name>(),
    )
        .join()
        .filter(|item| item.1.owner == *ecs.fetch::<Entity>()) {
        pack.entry(unobf_name_ecs(ecs, entity))
            .and_modify(|(count, _e)| {
                *count += 1;
            })
            .or_insert((1, entity));
    }
    let mut result: String = Default::default();
    for item in pack {
        let name = if item.1.0 != 1 {
            format!("{} {}", item.1.0, item.0.1)
        } else {
            // TODO: Get correct article (a/an/some) here, write a fn for it.
            item.0.0
        };
        result.push_str(&format!("- {}\n", name));
    }
    result.push_str("\n");
    return result;
}

fn draw_events_list() -> String {
    // Initialise default (empty) string
    let mut result: String = Default::default();
    // Get lock on events mutex
    let lock = gamelog::EVENTS.lock().unwrap();
    // Collect all keys, and sort in ascending value (by turn count)
    let mut sorted_keys: Vec<u32> = lock
        .keys()
        .map(|k| *k)
        .collect();
    sorted_keys.sort();
    // Iterate through sorted keys, looking for corresponding values, and append on newline
    for key in sorted_keys {
        if let Some(value) = lock.get(&key) {
            result.push_str(&format!("{:<4} | ", key));
            for (i, event) in value.iter().enumerate() {
                if i > 0 {
                    result.push_str(&format!("; {}", event.to_lowercase()));
                } else {
                    result.push_str(&format!("{}", event));
                }
            }
            result.push_str("\n");
        }
    }

    return result;
}
