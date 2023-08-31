use specs::prelude::*;
use crate::gui;
use crate::map::TileType;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput, // Player's turn
    PreRun,
    Ticking, // Tick systems
    ShowCheatMenu, // Teleport, godmode, etc. - for debugging
    ShowInventory,
    ShowDropItem,
    ShowRemoveItem,
    ShowTargeting {
        x: i32,
        y: i32,
        range: i32,
        item: Entity,
        aoe: i32,
    },
    ShowRemoveCurse,
    ShowIdentify,
    ActionWithDirection {
        function: fn(i: i32, j: i32, ecs: &mut World) -> RunState,
    },
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    CharacterCreation {
        ancestry: gui::Ancestry,
        class: gui::Class,
    },
    SaveGame,
    GameOver,
    GoToLevel(i32, TileType),
    HelpScreen,
    MagicMapReveal {
        row: i32,
        cursed: bool,
    }, // Animates magic mapping effect
    MapGeneration,
    Farlook {
        x: i32,
        y: i32,
    },
}
