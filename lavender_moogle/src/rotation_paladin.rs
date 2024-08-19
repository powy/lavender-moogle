// rotation_paladin.rs
// Rotation logic for paladin job

use log::error;

use crate::types::CharacterDetails;
use crate::spell_actions;
use crate::memory;


pub fn update_next_spellcast(character_details: &mut CharacterDetails) {
    if let Some(job_details) = character_details.get_mut("PLD") {
        // update status
        // check oGCD
        let mut next_spell_name = "";

        // No clipping?

        if spell_actions::ogcd_end_before_next_gcd(job_details)
        {
            if spell_actions::is_spell_available_now(job_details, "Fight or Flight") {
                next_spell_name = "Fight or Flight";
            } else if spell_actions::is_spell_available_now(job_details, "Imperator")
                    && !spell_actions::spell_will_be_available_soon(job_details, "Fight or Flight", 10000) {
                next_spell_name = "Imperator";
            } else if spell_actions::is_spell_available_now(job_details, "Circle of Scorn") {
                next_spell_name = "Circle of Scorn";
            } else if spell_actions::is_spell_available_now(job_details, "Expiacion") {
                next_spell_name = "Expiacion";
            } else if spell_actions::is_status_active_now(job_details, "Blade of Honor Ready") {
                next_spell_name = "Blade of Honor";
            }
        }

        // No oGCD selected, prepare next GCD

        // check expiring GCD        
        if next_spell_name == "" {
            
            if spell_actions::is_status_expiring(job_details, "Confiteor Ready") {
                next_spell_name = "Confiteor";
            } else if spell_actions::is_status_expiring(job_details, "Blade of Faith Ready") {
                next_spell_name = "Blade of Faith";
            } else if spell_actions::is_status_expiring(job_details, "Blade of Truth Ready") {
                next_spell_name = "Blade of Truth";
            } else if spell_actions::is_status_expiring(job_details, "Blade of Valor Ready") {
                next_spell_name = "Blade of Valor";
            } else if spell_actions::is_status_expiring(job_details, "Goring Blade Ready") {
                    next_spell_name = "Goring Blade";
            } else if spell_actions::is_status_expiring(job_details, "Sepulchre Ready") {
                next_spell_name = "Sepulchre";
            } else if spell_actions::is_status_expiring(job_details, "Supplication Ready") {
                next_spell_name = "Supplication";
            } else if spell_actions::is_status_expiring(job_details, "Atonement Ready") {
                next_spell_name = "Atonement";
            } else if spell_actions::is_status_expiring(job_details,"Royal Authority Ready") {
                next_spell_name = "Royal Authority";
            } else if spell_actions::is_status_expiring(job_details, "Divine Might") {
                next_spell_name = "Holy Spirit";
            }
        }

        // check standart GCD
        if next_spell_name == "" {
            
            if spell_actions::is_status_active_next_gcd(job_details, "Confiteor Ready") {
                next_spell_name = "Confiteor";
            } else if spell_actions::is_status_active_next_gcd(job_details, "Blade of Faith Ready") {
                next_spell_name = "Blade of Faith";
            } else if spell_actions::is_status_active_next_gcd(job_details, "Blade of Truth Ready") {
                next_spell_name = "Blade of Truth";
            } else if spell_actions::is_status_active_next_gcd(job_details, "Blade of Valor Ready") {
                next_spell_name = "Blade of Valor";
            } else if spell_actions::is_status_active_next_gcd(job_details, "Goring Blade Ready") {
                next_spell_name = "Goring Blade";
            } else if spell_actions::is_status_active_next_gcd(job_details, "Sepulchre Ready") {
                next_spell_name = "Sepulchre";
            } else if spell_actions::is_status_active_next_gcd(job_details, "Supplication Ready") {
                next_spell_name = "Supplication";
            } else if spell_actions::is_status_active_next_gcd(job_details, "Atonement Ready") {
                next_spell_name = "Atonement";
            } else if spell_actions::is_status_active_next_gcd(job_details,"Royal Authority Ready") {
                // Special case: Holy spirit only if it will disappear by royal authority
                // Ortherwise it is kept as long as possible for mobility uptime
                if spell_actions::is_status_active_next_gcd(job_details,"Divine Might") {
                    next_spell_name = "Holy Spirit";
                } else {
                    next_spell_name = "Royal Authority";
                }
            } else if spell_actions::is_status_active_next_gcd(job_details, "Riot Blade Ready") {                
                next_spell_name = "Riot Blade";
            } else {
                next_spell_name = "Fast Blade";
            }
        }
        if next_spell_name != "" {
            memory::communicate_next_spellcast(job_details, next_spell_name);
        }
    } else {
        error!("JOB not found in spell book");
    }
}
