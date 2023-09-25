mod builder;
pub use builder::*;
mod logstore;
use logstore::*;
pub use logstore::{ LOG, clear_log, clone_log, render, render_log, restore_log, setup_log };
mod events;
pub use events::*;

use serde::{ Deserialize, Serialize };
use bracket_lib::prelude::*;
#[derive(Serialize, Deserialize, Clone)]
pub struct LogFragment {
    pub colour: RGB,
    pub text: String,
}
