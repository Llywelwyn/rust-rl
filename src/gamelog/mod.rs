use rltk::prelude::*;

mod builder;
pub use builder::*;
mod logstore;
use logstore::*;
pub use logstore::{ clear_log, clone_log, print_log, restore_log, setup_log };
mod events;
pub use events::*;

use serde::{ Deserialize, Serialize };
#[derive(Serialize, Deserialize, Clone)]
pub struct LogFragment {
    pub colour: RGB,
    pub text: String,
}
