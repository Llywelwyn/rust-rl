use super::{ append_entry, LogFragment };
use crate::BUC;
use bracket_lib::prelude::*;

pub struct Logger {
    current_colour: RGB,
    fragments: Vec<LogFragment>,
}

impl Logger {
    /// Creates a blank builder for making message log entries.
    pub fn new() -> Self {
        Logger { current_colour: RGB::named(WHITE), fragments: Vec::new() }
    }

    /// Sets the colour of the current message logger.
    pub fn colour(mut self, colour: (u8, u8, u8)) -> Self {
        self.current_colour = RGB::named(colour);
        return self;
    }

    /// Appends text in the current colour to the current message logger.
    pub fn append<T: ToString>(mut self, text: T) -> Self {
        let mut text_with_space = text.to_string();
        text_with_space.push_str(" ");
        self.fragments.push(LogFragment { colour: self.current_colour, text: text_with_space });
        return self;
    }

    /// Appends text in the current colour to the current message logger, with no space.
    #[allow(dead_code)]
    pub fn append_n<T: ToString>(mut self, text: T) -> Self {
        self.fragments.push(LogFragment { colour: self.current_colour, text: text.to_string() });
        return self;
    }

    /// Appends a period to the current message logger.
    pub fn period(mut self) -> Self {
        self.fragments.push(LogFragment { colour: self.current_colour, text: ". ".to_string() });
        return self;
    }

    pub fn buc<T: ToString>(mut self, buc: BUC, cursed: Option<T>, blessed: Option<T>) -> Self {
        if buc == BUC::Cursed && cursed.is_some() {
            self.fragments.push(LogFragment {
                colour: RGB::named(SALMON),
                text: cursed.unwrap().to_string(),
            });
            self.fragments.push(LogFragment {
                colour: self.current_colour,
                text: ". ".to_string(),
            });
        } else if buc == BUC::Blessed && blessed.is_some() {
            self.fragments.push(LogFragment {
                colour: RGB::named(CYAN),
                text: blessed.unwrap().to_string(),
            });
            self.fragments.push(LogFragment {
                colour: self.current_colour,
                text: ". ".to_string(),
            });
        }
        return self;
    }

    /// Pushes the finished log entry.
    pub fn log(self) {
        return append_entry(self.fragments);
    }
}
