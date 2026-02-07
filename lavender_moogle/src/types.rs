use chrono::{DateTime, Duration, Utc};
use log::debug;
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
    pub max_charges: i32,
    pub charge_timers: Vec<DateTime<Utc>>,
    pub last_activation_time: DateTime<Utc>, // When status was last activated
    pub last_cast_time: DateTime<Utc>,       // When spell was last cast
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
    pub opener: bool,
    pub dump_resources: bool,
    pub burst_allowed: bool,
    pub last_activation_spell: String, // Name of the last spell that activated a status
    pub last_cast_spell: String,       // Name of the last spell that was cast
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
        job_details.opener = true;
        job_details.dump_resources = false;
        job_details.burst_allowed = true;
        job_details.last_activation_spell = String::new();
        job_details.last_cast_spell = String::new();
        for spell in job_details.spells.values_mut() {
            spell.active_until = now_minus_500ms;
            spell.unavailable_before = now_minus_500ms;
            spell.charge_timers.clear();
            spell.last_activation_time = Utc::now();
            spell.last_cast_time = Utc::now();
        }
    }
}

pub fn set_dump_resources(character_details: &mut CharacterDetails, value: bool) {
    debug!("set_dump_resources called with value: {}", value);
    for (job_name, job_details) in character_details.iter_mut() {
        debug!("  Setting dump_resources={} for job: {}", value, job_name);
        job_details.dump_resources = value;
    }
}

pub fn set_burst_allowed(character_details: &mut CharacterDetails, value: bool) {
    debug!("set_burst_allowed called with value: {}", value);
    for (job_name, job_details) in character_details.iter_mut() {
        debug!("  Setting burst_allowed={} for job: {}", value, job_name);
        job_details.burst_allowed = value;
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
    First = 0,
    Second = 1,
    Third = 2,
    Fourth = 3,
}

impl TryFrom<usize> for SpellcastType {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SpellcastType::First),
            1 => Ok(SpellcastType::Second),
            2 => Ok(SpellcastType::Third),
            3 => Ok(SpellcastType::Fourth),
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
            (true, true) => Self::Fourth,
            (true, false) => Self::Second,
            (false, true) => Self::Third,
            (false, false) => Self::First,
        }
    }
}
