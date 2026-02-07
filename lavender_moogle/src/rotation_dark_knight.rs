// rotation_dark_knight.rs
// Rotation logic for dark knight job

use log::error;

use crate::memory;
use crate::spell;
use crate::types::{CharacterDetails, JobDetails, SpellcastType};

pub fn update_next_spellcast(character_details: &mut CharacterDetails, spell_cast_type: SpellcastType, aoe: bool, alternate: bool) {
    if let Some(job_details) = character_details.get_mut("DRK") {
        // update status
        // check oGCD
        let mut next_spell_name = "";
        // let burst_allowed = spell::burst_allowed(job_details);
        // let delayed_burst = !burst_allowed;

        // update stack min / max

        // Shadowbringer
        if job_details.stacks_a > 2 {
            job_details.stacks_a = 2;
        } else if job_details.stacks_a < 0 {
            job_details.stacks_a = 0;
        }

        // Edge of Shadow
        if job_details.stacks_b > 3 {
            job_details.stacks_b = 3;
        } else if job_details.stacks_b < 0 {
            job_details.stacks_b = 0;
        }

        // adjustment for blood gauge (secondary_bar)
        if job_details.secondary_bar > 250 {
            job_details.secondary_bar = 250;
        } else if job_details.secondary_bar < 0 {
            job_details.secondary_bar = 0;
        }

        // No clipping?

        if spell::ogcd_end_before_next_gcd(job_details) {
            // === Trigger buffs
            if spell::is_spell_available_now(job_details, "Living Shadow") {
                next_spell_name = "Living Shadow";
            } else if spell::is_spell_available_now(job_details, "Delirium") {
                next_spell_name = "Delirium";
            }

            // === Manage burst window
            // In delirium + clone but no immediate double weave
            if next_spell_name == ""
                && spell::is_status_active_now(job_details, "2m Burst")
                && spell::is_status_active_now(job_details, "1m Burst")
                && spell::spell_will_be_available_soon(job_details, "Delirium", 58000)
            {
                // Salted Earth
                if spell::is_spell_available_now(job_details, "Salted Earth") {
                    next_spell_name = "Salted Earth";
                // Edge of Shadow
                } else if spell::is_spell_available_now(job_details, "Edge of Shadow")
                    && job_details.stacks_b > 0
                {
                    next_spell_name = "Edge of Shadow";
                // First shadowbringer charge
                } else if spell::is_spell_available_now(job_details, "Shadowbringer")
                    && job_details.stacks_a > 1
                {
                    next_spell_name = "Shadowbringer";
                // Carve and spit
                } else if spell::is_spell_available_now(job_details, "Carve and Spit") {
                    next_spell_name = "Carve and Spit";
                // Second shadowbringer charge
                } else if spell::is_spell_available_now(job_details, "Shadowbringer")
                    && job_details.stacks_a > 0
                {
                    next_spell_name = "Shadowbringer";
                }
            }

            // classic oGCD
            if next_spell_name == "" {
                // Avoid same weave for salt and darkness combo
                if spell::is_status_active_now(job_details, "Salt and Darkness Ready")
                    && spell::spell_will_be_available_soon(job_details, "Salted Earth", 88000)
                {
                    next_spell_name = "Salt and Darkness";
                } else if spell::is_spell_available_now(job_details, "Carve and Spit")
                    && !spell::spell_will_be_available_soon(job_details, "Delirium", 10000)
                    && !spell::is_status_active_now(job_details, "2m Burst")
                {
                    // If in 2m burst, use the above prioriy list instead
                    next_spell_name = "Carve and Spit";
                } else if spell::is_spell_available_now(job_details, "Salted Earth")
                    && !spell::spell_will_be_available_soon(job_details, "Delirium", 10000)
                    && !spell::is_status_active_now(job_details, "2m Burst")
                {
                    // If in 2m burst, use the above prioriy list instead
                    // To be noted, it is usually only for the opener
                    next_spell_name = "Salted Earth";
                }
            }

            // safeguard no more EoS
            if next_spell_name == "" && !spell::is_status_active_now(job_details, "Recent EoS") {
                next_spell_name = "Edge of Shadow";
            }
        }

        // No oGCD selected, prepare next GCD

        // check expiring GCD
        if next_spell_name == "" {
            if spell::is_status_expiring(job_details, "Souleater Ready") {
                next_spell_name = "Souleater";
            } else if spell::is_status_expiring(job_details, "Syphon Strike Ready") {
                next_spell_name = "Syphon Strike";
            }
        }

        // check burst GCD
        if next_spell_name == ""
            && spell::is_status_active_next_gcd(job_details, "2m Burst")
            && spell::is_status_active_next_gcd(job_details, "1m Burst")
        {
            if spell::is_status_active_next_gcd(job_details, "Scorn") {
                next_spell_name = "Disesteem";
            } else if job_details.secondary_bar >= 50 {
                next_spell_name = "Bloodspiller";
            }
        }

        // check standart GCD
        // Not in burst window - avoid overcap blood or classic combo
        if next_spell_name == "" {
            if spell::is_status_active_next_gcd(job_details, "Souleater Ready") {
                if job_details.secondary_bar >= 70 {
                    next_spell_name = "Bloodspiller";
                } else {
                    next_spell_name = "Souleater";
                }
            } else if spell::is_status_active_next_gcd(job_details, "Syphon Strike Ready") {
                next_spell_name = "Syphon Strike";
            } else {
                next_spell_name = "Hard Slash";
            }
        }
        if next_spell_name != "" {
            memory::communicate_next_spellcast(job_details, spell_cast_type, next_spell_name, aoe, alternate);
        }
    } else {
        error!("JOB not found in spell book");
    }
}

pub fn apply_spell_additional_effects(spell_name: &str, _job_details: &mut JobDetails) {
    match spell_name {
        _ => {}
    }
}
