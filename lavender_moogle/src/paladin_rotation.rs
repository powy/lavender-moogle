use crate::types::{SpellBook};
// use crate::types::{SpellDescription, JobSpells, SpellBook};
use crate::spell_actions;
use log::{debug, error};
use once_cell::sync::Lazy;
use std::sync::Mutex;

static NEXT_SPELL_STR: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::from("none")));


pub fn update_next_spellcast_paladin(spell_book: &mut SpellBook) {
    if let Some(job_spells) = spell_book.get_mut("PLD") {
        // update status
        // check oGCD

        // check GCD

        // Royal Authority
        if spell_actions::is_status_active_next_gcd(job_spells,"Royal Authority Ready") {
            update_next_spellcast("Royal Authority");
        } else if spell_actions::is_status_active_next_gcd(job_spells, "Riot Blade Ready") {
            update_next_spellcast("Riot Blade");
        } else {
            update_next_spellcast("Fast Blade");
        }



    } else {
        error!("PLD not found in spell book");
    }
}

// FIXME unsafe
pub fn update_next_spellcast(spell_name: &str) {
    let mut next_spell = NEXT_SPELL_STR.lock().unwrap();
    if *next_spell != spell_name {
        debug!("Updating next spellcast to {}", spell_name);
        *next_spell = spell_name.to_string(); // copy
    }
    // else {  
    //     debug!("Next spellcast is already {}", spell_name);
    // }
}