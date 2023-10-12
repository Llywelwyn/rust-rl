use bracket_lib::prelude::*;
use notan::prelude::*;
use specs::prelude::*;
use std::sync::Mutex;
use std::collections::HashMap;
use super::{ EffectSpawner, EffectType, Targets, add_effect };
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

pub fn stop(app: &mut App) {
    let mut ambience = AMBIENCE.lock().unwrap();
    if let Some(old) = ambience.take() {
        app.audio.stop(&old);
    }
}

pub fn ambience(sound: &str) {
    add_effect(None, EffectType::Sound { sound: sound.to_string() }, Targets::Tile { target: 0 })
}

pub fn replace_ambience(app: &mut App, sound: &Sound) {
    let mut ambience = AMBIENCE.lock().unwrap();
    if let Some(old) = ambience.take() {
        app.audio.stop(&old);
    }
    *ambience = Some(sound.clone());
}

pub fn init_sounds(app: &mut App) {
    let sound_data: &[(&str, &[u8], AudioType)] = &[
        // (key, file_path, audiotype)
        ("a_relax", include_bytes!("../../resources/sounds/amb/relaxed.ogg"), AudioType::Ambient),
        ("d_blocked1", include_bytes!("../../resources/sounds/door/blocked1.wav"), AudioType::SFX),
        ("d_blocked2", include_bytes!("../../resources/sounds/door/blocked2.wav"), AudioType::SFX),
        ("d_blocked3", include_bytes!("../../resources/sounds/door/blocked3.wav"), AudioType::SFX),
        ("d_open1", include_bytes!("../../resources/sounds/door/open1.wav"), AudioType::SFX),
        ("d_open2", include_bytes!("../../resources/sounds/door/open2.wav"), AudioType::SFX),
        ("d_open3", include_bytes!("../../resources/sounds/door/open3.wav"), AudioType::SFX),
        ("d_close1", include_bytes!("../../resources/sounds/door/close1.wav"), AudioType::SFX),
        ("d_close2", include_bytes!("../../resources/sounds/door/close2.wav"), AudioType::SFX),
        ("d_close3", include_bytes!("../../resources/sounds/door/close3.wav"), AudioType::SFX),
    ];
    let mut sounds = SOUNDS.lock().unwrap();
    for (k, bytes, audiotype) in sound_data {
        sounds.insert(k.to_string(), (app.audio.create_source(bytes).unwrap(), *audiotype));
    }
}

pub fn set_volume(vol: f32) {
    let mut volume = VOLUME.lock().unwrap();
    *volume = vol;
}

pub fn clean(app: &mut App) {
    app.audio.clean();
}

// Shorthand functions for adding generic, frequent SFX to the effect queue.
pub fn door_open(idx: usize) {
    let mut rng = RandomNumberGenerator::new();
    let sound = (
        match rng.range(0, 3) {
            0 => "d_open1",
            1 => "d_open2",
            _ => "d_open3",
        }
    ).to_string();
    super::add_effect(None, EffectType::Sound { sound }, Targets::Tile { target: idx });
}
pub fn door_resist(idx: usize) {
    let mut rng = RandomNumberGenerator::new();
    let sound = (
        match rng.range(0, 3) {
            0 => "d_blocked1",
            1 => "d_blocked2",
            _ => "d_blocked3",
        }
    ).to_string();
    add_effect(None, EffectType::Sound { sound }, Targets::Tile { target: idx });
}
pub fn door_close(idx: usize) {
    let mut rng = RandomNumberGenerator::new();
    let sound = (
        match rng.range(0, 3) {
            0 => "d_close1",
            1 => "d_close2",
            _ => "d_close3",
        }
    ).to_string();
    add_effect(None, EffectType::Sound { sound }, Targets::Tile { target: idx });
}
