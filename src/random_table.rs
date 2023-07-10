use rltk::RandomNumberGenerator;

// FIXME: note to self,
// passing around strings here is super inefficient, so this is
// a good place to look for optimisation. Using a unique ID here
// for every entry on the table and returning that instead of the
// name would be helpful. That said, these tables see low enough
// use that reducing readability and confusing things right now
// seems like a waste of time.
pub struct RandomEntry {
    name: String,
    weight: i32,
}

impl RandomEntry {
    pub fn new<S: ToString>(name: S, weight: i32) -> RandomEntry {
        RandomEntry { name: name.to_string(), weight }
    }
}

#[derive(Default)]
pub struct RandomTable {
    entries: Vec<RandomEntry>,
    total_weight: i32,
}

impl RandomTable {
    /// Creates a new, blank RandomTable
    pub fn new() -> RandomTable {
        return RandomTable { entries: Vec::new(), total_weight: 0 };
    }

    /// Adds an entry to an existing RandomTable
    pub fn add<S: ToString>(mut self, name: S, weight: i32) -> RandomTable {
        self.total_weight += weight;
        self.entries.push(RandomEntry::new(name.to_string(), weight));
        return self;
    }

    /// Rolls on an existing RandomTable
    pub fn roll(&self, rng: &mut RandomNumberGenerator) -> String {
        // If the table has no weight, return nothing.
        if self.total_weight == 0 {
            return "None".to_string();
        }

        // If the table has weight, roll a die, and iterate through
        // every index on the RandomTable. If the roll is below the
        // weight of the current index, return it - otherwise, reduce
        // the roll by the weight and test the next entry.
        let mut roll = rng.roll_dice(1, self.total_weight) - 1;
        let mut index: usize = 0;
        while roll > 0 {
            if roll < self.entries[index].weight {
                return self.entries[index].name.clone();
            }

            roll -= self.entries[index].weight;
            index += 1;
        }

        // If the rolling fails to produce anything (i.e. there is no
        // item on the table with a weight large to be spawned
        // by the roll) then return nothing.
        return "None".to_string();
    }
}
