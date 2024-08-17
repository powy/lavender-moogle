// Type describing a spell, its caracteristic, its key binding
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpellDescription {
    pub name: String,
    pub modifier: char,
    pub key: char,
    pub spell_type: String,  // 'type' is a reserved keyword in Rust, so I renamed it to 'spell_type'
    pub duration: i32,
    pub cooldown: i32,
    pub give_buffs: Vec<String>,
    pub remove_buffs: Vec<String>,
    pub available_until: DateTime<Utc>,
}

// Contains all the spells organised by job - spell name - spell description
// Type alias for a dictionary of dictionaries
pub type SpellBook = HashMap<String, HashMap<String, SpellDescription>>;

