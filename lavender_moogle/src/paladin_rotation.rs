use crate::types::{SpellBook, JobSpells};
// use crate::types::{SpellDescription, JobSpells, SpellBook};
use crate::spell_actions;
use crate::memory;
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
            update_next_spellcast(job_spells, "Royal Authority");
        } else if spell_actions::is_status_active_next_gcd(job_spells, "Riot Blade Ready") {
            update_next_spellcast(job_spells, "Riot Blade");
        } else {
            update_next_spellcast(job_spells, "Fast Blade");
        }



    } else {
        error!("PLD not found in spell book");
    }
}

// FIXME unsafe
pub fn update_next_spellcast(job_spells: &mut JobSpells, spell_name: &str) {
    let mut saved_next_spell_name = NEXT_SPELL_STR.lock().unwrap();
    if *saved_next_spell_name != spell_name {
        debug!("Updating next spellcast to {}", spell_name);
        *saved_next_spell_name = spell_name.to_string(); // copy
        // if let Some(next_spell_description) = job_spells.get(spell_name).cloned() {
        //     memory::populate_shared_memory(next_spell_description);
        // } else {
        //     error!("Spell {} not found for shared memory", spell_name);
        // }
    }
}