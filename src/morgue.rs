use std::fs::{ File, create_dir_all };
use std::io::{ self, Write };
use std::time::SystemTime;
use specs::prelude::*;
use rltk::prelude::*;

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
    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    let morgue_info = format!(
        r#"╔══════════════════════════════════════════════════════════════╗
║ Level 1, human wizard                                        ║
╚══════════════════════════════════════════════════════════════╝"#
    );
    let file_name = format!("{}/{}-{}-{}.txt", morgue_dir, "human", "wizard", timestamp);
    let mut file = File::create(&file_name)?; // Open/create morgue file
    file.write_all(morgue_info.as_bytes())?;
    Ok(())
}
