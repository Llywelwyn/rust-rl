use bracket_lib::prelude::*;
use notan::prelude::*;
use specs::prelude::*;
use std::sync::Mutex;
use std::collections::HashMap;
use super::{ EffectSpawner, EffectType };
use crate::Map;

lazy_static::lazy_static! {
    pub static ref SOUNDS: Mutex<HashMap<String, (AudioSource, AudioType)>> = Mutex::new(HashMap::new());
    pub static ref VOLUME: Mutex<f32> = Mutex::new(1.0);
    pub static ref AMBIENCE: Mutex<Option<Sound>> = Mutex::new(None);
}

#[derive(PartialEq, Copy, Clone)]
pub enum AudioType {
    Ambient,
    SFX,
}

const AMBIENCE_VOL_MUL: f32 = 0.8;
const SFX_VOL_MUL: f32 = 1.0;

pub fn play_sound(app: &mut App, ecs: &mut World, effect: &EffectSpawner, target: usize) {
    // Extract sound from the EffectType, or panic if we somehow called this with the wrong effect.
    let sound = if let EffectType::Sound { sound } = &effect.effect_type {
        sound
    } else {
        unreachable!("add_intrinsic() called with the wrong EffectType")
    };
    // Fetch all the relevant precursors.
    let sounds = SOUNDS.lock().unwrap();
    let volume = VOLUME.lock().unwrap();
    let source = sounds.get(sound).unwrap();
    let (vol, repeat) = match source.1 {
        AudioType::Ambient => (*volume * AMBIENCE_VOL_MUL, true),
        AudioType::SFX => {
            let map = ecs.fetch::<Map>();
            let ppos = ecs.fetch::<Point>();
            // Calc distance from player to target.
            let dist = DistanceAlg::PythagorasSquared.distance2d(
                *ppos,
                Point::new((target as i32) % map.width, (target as i32) / map.width)
            );
            // Play sound at volume proportional to distance.
            (*volume * SFX_VOL_MUL * (1.0 - (dist as f32) / 14.0), false)
        }
    };
    // Play the sound.
    let sound: Sound = app.audio.play_sound(&source.0, vol, repeat);
    if repeat {
        replace_ambience(app, &sound);
    }
}

pub fn replace_ambience(app: &mut App, sound: &Sound) {
    let mut ambience = AMBIENCE.lock().unwrap();
    if let Some(old) = ambience.take() {
        app.audio.stop(&old);
    }
    *ambience = Some(sound.clone());
}

pub fn init_sounds(app: &mut App) {
    let list: Vec<(&str, (&[u8], AudioType))> = vec![
        //key, (bytes, type) - audiotype determines final volume, looping, etc.
        ("hit", (include_bytes!("../../resources/sounds/hit.wav"), AudioType::SFX))
    ];
    let mut sounds = SOUNDS.lock().unwrap();
    for (k, (bytes, audiotype)) in list.iter() {
        sounds.insert(k.to_string(), (app.audio.create_source(bytes).unwrap(), *audiotype));
    }
}

pub fn set_volume(vol: f32) {
    let mut volume = VOLUME.lock().unwrap();
    *volume = vol;
}
