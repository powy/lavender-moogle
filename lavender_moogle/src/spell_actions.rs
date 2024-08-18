use chrono::{DateTime, Utc};
use crate::types::{SpellDescription, JobSpells, SpellBook};
use log::{debug, error};


pub fn apply_spell_cast(spell_book: &mut SpellBook, job_name: &str, spell_name: &str) {
    if let Some(job_spells) = spell_book.get_mut(job_name) {
        if let Some(spell_description) = job_spells.get(spell_name).cloned() {
       
            debug!("Action for ability: {}", spell_name);
            // Apply self CD -- require mutable
            apply_spell_CD(job_spells, spell_name);
            apply_gcd_on_cast(job_spells, &spell_description);
            // Important to remove status before activating them !
            deactivate_related_status_on_cast(job_spells, &spell_description);
            apply_new_status_on_cast(job_spells, &spell_description);
        } else {
            debug!("Spell {} not found for job {}", spell_name, job_name);
        }
    } else {
        error!("Job {} not found in spell book", job_name);
    }
}

pub fn apply_spell_CD(job_spells: &mut JobSpells, spell_name: &str) {
    if let Some(spell_description) = job_spells.get_mut(spell_name) {
        if spell_description.cooldown > 0 {
            spell_description.unavaible_before = calculate_future_time(spell_description.cooldown, 0);
            debug!("Apply CD for ability: {}, next avaibility {}", spell_description.name, spell_description.unavaible_before);
        }
    } else {
        error!("Spell {} not found", spell_name);
        return;
    }
}

pub fn apply_gcd_on_cast(job_spells: &mut JobSpells, spell_description: &SpellDescription) {
    if spell_description.spell_type == "GCD" {
        // Mutably borrow job_spells again after the first mutable borrow has ended
        activate_status(job_spells, "gcd unavailable", 0);
    }
}

pub fn activate_status(job_spells: &mut JobSpells, spell_name: &str, margin_time: i32) {
    if let Some(spell_description) = job_spells.get_mut(spell_name) {
        let duration = spell_description.duration;
        spell_description.active_until = calculate_future_time(duration, margin_time);
        debug!("Status {} activated for {} milliseconds, until {}", spell_name, duration, spell_description.active_until);
    } else {
        error!("Effect to activate {} not found in spell book", spell_name);
    }
}

pub fn deactivate_status(job_spells: &mut JobSpells, spell_name: &str) {
    if let Some(spell_description) = job_spells.get_mut(spell_name) {
        spell_description.active_until = calculate_future_time(-100, 0);
        debug!("Status {} deactivated", spell_name);
    } else {
        error!("Effect to deactivate {} not found in spell book", spell_name);
    }
}

pub fn apply_new_status_on_cast(job_spells: &mut JobSpells, spell_description: &SpellDescription) {
    for status_name in &spell_description.give_status {
        activate_status(job_spells, status_name, 0);
    }
}

pub fn deactivate_related_status_on_cast(job_spells: &mut JobSpells, spell_description: &SpellDescription) {
    for status_name in &spell_description.remove_status {
        deactivate_status(job_spells, status_name);
    }
}

pub fn calculate_future_time(duration_value: i32, margin_time: i32) -> DateTime<Utc> {
    let current_time = Utc::now();
    let duration = chrono::Duration::milliseconds(duration_value.into());
    let margin = chrono::Duration::milliseconds(margin_time.into());
    current_time + duration + margin
}

// FIXME Implementation for resetting all CD and status
pub fn reset(spell_book: &mut SpellBook) {
    debug!("RESET");
}

