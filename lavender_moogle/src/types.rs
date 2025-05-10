use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpellDetails {
    pub name: String,
    pub modifier: char,
    pub key: char,
    pub spell_type: String,
    pub duration: i32,
    pub cooldown: i32,
    pub give_status: Vec<String>,
    pub remove_status: Vec<String>,
    pub active_until: DateTime<Utc>,
    pub unavailable_before: DateTime<Utc>,
    pub mana: i32,
    pub secondary_bar: i32,
    pub tertiary_bar: i32,
    pub stacks_a: i32,
    pub stacks_b: i32,
}

// Represent all spells associated with a specific job
// pub type JobDetails = HashMap<String, SpellDetails>;
pub struct JobDetails {
    pub spells: HashMap<String, SpellDetails>,
    pub mana: i32,
    pub secondary_bar: i32,
    pub tertiary_bar: i32,
    pub stacks_a: i32,
    pub stacks_b: i32,
    pub weaves: i32,
}

// Used to use JobDetails like a table of spells
impl JobDetails {
    // Method to get a mutable reference to a spell by its name
    pub fn get_spell(&mut self, spell_name: &str) -> Option<&mut SpellDetails> {
        self.spells.get_mut(spell_name)
    }

    // Method to insert a spell into the HashMap
    pub fn insert_spell(&mut self, name: String, spell_details: SpellDetails) {
        self.spells.insert(name, spell_details);
    }
}

// Contains all the spells organised by job - spell name - spell description
// Type alias for a dictionary of dictionaries
pub type CharacterDetails = HashMap<String, JobDetails>;

pub fn reset_character_status(character_details: &mut CharacterDetails) {
    let now_minus_500ms = Utc::now() - Duration::milliseconds(500);

    for job_details in character_details.values_mut() {
        job_details.mana = 0;
        job_details.secondary_bar = 0;
        job_details.tertiary_bar = 0;
        job_details.stacks_a = 0;
        job_details.stacks_b = 0;
        job_details.weaves = 0;
        for spell in job_details.spells.values_mut() {
            spell.active_until = now_minus_500ms;
            spell.unavailable_before = now_minus_500ms;
        }
    }
}

// impl CharacterDetails {
//     pub fn reset_timestamps(&mut self) {
//         let now_minus_500ms = Utc::now() - Duration::milliseconds(500);

//         for job_details in self.values_mut() {
//             for spell in job_details.spells.values_mut() {
//                 spell.active_until = now_minus_500ms;
//                 spell.unavailable_before = now_minus_500ms;
//             }
//         }
//     }
// }

// Spell Cast type is used to determine which type of action/rotation we are working on
// The system allows for 4 different rotations, usable changing the modifier (alt, shift, ctrl)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SpellcastType {
    MonoStandard = 0,
    MonoAlternate = 1,
    AoeStandard = 2,
    AoeAlternate = 3,
}

impl TryFrom<usize> for SpellcastType {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SpellcastType::MonoStandard),
            1 => Ok(SpellcastType::MonoAlternate),
            2 => Ok(SpellcastType::AoeStandard),
            3 => Ok(SpellcastType::AoeAlternate),
            _ => Err(()),
        }
    }
}

impl SpellcastType {
    pub fn to_position(self) -> usize {
        self as usize
    }

    pub fn from_flags(alternate: bool, aoe: bool) -> Self {
        match (alternate, aoe) {
            (true, true) => Self::AoeAlternate,
            (true, false) => Self::MonoAlternate,
            (false, true) => Self::AoeStandard,
            (false, false) => Self::MonoStandard,
        }
    }
}
