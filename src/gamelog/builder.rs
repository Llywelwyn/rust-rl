use super::{append_entry, LogFragment};
use rltk::prelude::*;

pub struct Logger {
    current_colour: RGB,
    fragments: Vec<LogFragment>,
}

impl Logger {
    pub fn new() -> Self {
        Logger { current_colour: RGB::named(rltk::WHITE), fragments: Vec::new() }
    }

    pub fn colour(mut self, colour: (u8, u8, u8)) -> Self {
        self.current_colour = RGB::named(colour);
        return self;
    }

    pub fn append<T: ToString>(mut self, text: T) -> Self {
        self.fragments.push(LogFragment { colour: self.current_colour, text: text.to_string() });
        return self;
    }

    pub fn log(self) {
        return append_entry(self.fragments);
    }

    pub fn npc_name<T: ToString>(mut self, text: T) -> Self {
        self.fragments.push(LogFragment { colour: RGB::named(rltk::YELLOW), text: text.to_string() });
        return self;
    }

    pub fn item_name<T: ToString>(mut self, text: T) -> Self {
        self.fragments.push(LogFragment { colour: RGB::named(rltk::CYAN), text: text.to_string() });
        return self;
    }

    pub fn damage(mut self, damage: i32) -> Self {
        self.fragments.push(LogFragment { colour: RGB::named(rltk::RED), text: format!("{}", damage).to_string() });
        return self;
    }
}
