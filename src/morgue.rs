use std::fs::{ File, create_dir_all };
use std::io::{ self, Write };
use std::time::SystemTime;
use crate::components::*;
use crate::gui::{ Class, Ancestry, unobf_name_ecs };
use specs::prelude::*;
use rltk::prelude::*;
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
pub fn create_morgue_file(_ecs: &World) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_morgue_file(ecs: &World) {
    let morgue_dir = "morgue";
    if let Err(err) = create_dir_all(&morgue_dir) {
        console::log(format!("Unable to create the directory (/{}): {}", morgue_dir, err));
    }
    if let Err(err) = write_morgue_file(ecs, &morgue_dir) {
        console::log(format!("Unable to write the morgue file: {}", err));
    }
}

fn write_morgue_file(ecs: &World, morgue_dir: &str) -> Result<(), io::Error> {
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
    let attrs = ecs.read_storage::<Attributes>();
    let attr = attrs.get(*e).unwrap();
    let header = format!("{} {}, level {}/{}", &ancestry, &class, &pool.level, &pool.xp);
    morgue_info.push_str(&create_boxed_text(header.as_str(), None));
    morgue_info.push_str(&draw_tombstone(header.len()));
    morgue_info.push_str(
        format!(
            "HP {}/{}    MP {}/{}\n",
            pool.hit_points.current,
            pool.hit_points.max,
            pool.mana.current,
            pool.mana.max
        ).as_str()
    );
    morgue_info.push_str(&draw_attributes(attr));
    morgue_info.push_str(&create_boxed_text("Equipment", None));
    morgue_info.push_str(&draw_equipment(ecs));
    morgue_info.push_str(&create_boxed_text("Backpack", None));
    morgue_info.push_str(&draw_backpack(ecs));

    // Save to file
    let file_name = format!("{}/lv{}-{}-{}-{}.txt", morgue_dir, &pool.level, &ancestry, &class, get_timestamp());
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

fn draw_tombstone(len: usize) -> String {
    let pad = (len - 17) / 2;
    return format!(
        "\n{:^p$}    .-'~~~`-.\n{:^p$}  .'         `.\n{:^p$}  |  rest     |\n{:^p$}  |    in     |\n{:^p$}  |     peace |\n{:^p$}\\\\|           |//\n{:^p$}^^^^^^^^^^^^^^^^^{:^p$}\n\n",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        p = pad
    );
}

fn draw_attributes(attr: &Attributes) -> String {
    return format!(
        "\nSTR {:>2} ({:+})  CON {:>2} ({:+})  WIS {:>2} ({:+})\nDEX {:>2} ({:+})  INT {:>2} ({:+})  CHA {:>2} ({:+})\n\n",
        attr.strength.base + attr.strength.modifiers,
        attr.strength.bonus,
        attr.constitution.base + attr.constitution.modifiers,
        attr.constitution.bonus,
        attr.wisdom.base + attr.wisdom.modifiers,
        attr.wisdom.bonus,
        attr.dexterity.base + attr.dexterity.modifiers,
        attr.dexterity.bonus,
        attr.intelligence.base + attr.intelligence.modifiers,
        attr.intelligence.bonus,
        attr.charisma.base + attr.charisma.modifiers,
        attr.charisma.bonus
    );
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
        let name = if item.1 != 1 { unobf_name_ecs(ecs, item.0).1 } else { unobf_name_ecs(ecs, item.0).0 };
        result.push_str(&format!("{:>8} {}\n", slot, name));
    }
    result.push_str("\n");
    return result;
}

fn draw_backpack(ecs: &World) -> String {
    // Get all of the player's backpack.
    let mut pack: HashMap<(String, String), (i32, Entity)> = HashMap::new();
    for (entity, _bp, _n) in (&ecs.entities(), &ecs.read_storage::<InBackpack>(), &ecs.read_storage::<Name>())
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
