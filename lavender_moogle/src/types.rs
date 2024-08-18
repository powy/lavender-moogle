use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpellDescription {
    pub name: String,
    pub modifier: char,
    pub key: char,
    pub spell_type: String,
    pub duration: i32,
    pub cooldown: i32,
    pub give_status: Vec<String>,
    pub remove_status: Vec<String>,
    pub active_until: DateTime<Utc>,
    pub unavaible_before: DateTime<Utc>,
}

// Represent all spells associated with a specific job
pub type JobSpells = HashMap<String, SpellDescription>;

// Contains all the spells organised by job - spell name - spell description
// Type alias for a dictionary of dictionaries
pub type SpellBook = HashMap<String, JobSpells>;
