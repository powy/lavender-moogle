// rotation_paladin.rs
// Rotation logic for paladin job

use log::error;

use crate::memory;
use crate::spell;
use crate::types::{CharacterDetails, JobDetails};

pub fn update_next_spellcast(character_details: &mut CharacterDetails, aoe: bool, alternate: bool) {
    if let Some(job_details) = character_details.get_mut("PLD") {
        const MAX_WEAVE_NUMBER: i32 = 2;
        // update status
        // check oGCD
        let mut next_spell_name = "";
        // let burst_allowed = job_details.burst_allowed;
        // let delayed_burst = !burst_allowed;

        // No clipping?
        if (job_details.weaves < MAX_WEAVE_NUMBER) && spell::ogcd_end_before_next_gcd(job_details) {
            if spell::is_spell_available_now(job_details, "Fight or Flight")
                && job_details.burst_allowed
            {
                next_spell_name = "Fight or Flight";
            } else if spell::is_spell_available_now(job_details, "Imperator")
                && !spell::spell_will_be_available_soon(job_details, "Fight or Flight", 10000)
                && job_details.burst_allowed
            {
                next_spell_name = "Imperator";
            } else if spell::is_spell_available_now(job_details, "Circle of Scorn") {
                next_spell_name = "Circle of Scorn";
            } else if spell::is_spell_available_now(job_details, "Expiacion") {
                next_spell_name = "Expiacion";
            } else if spell::is_status_active_now(job_details, "Blade of Honor Ready") {
                next_spell_name = "Blade of Honor";
            }
        }

        // No oGCD selected, prepare next GCD

        // check expiring GCD // TBC - not worth to break a burst. Should not happen outside of burst
        // if next_spell_name == "" {
        //     if spell::is_status_expiring(job_details, "Confiteor Ready") {
        //         next_spell_name = "Confiteor";
        //     } else if spell::is_status_expiring(job_details, "Blade of Faith Ready") {
        //         next_spell_name = "Blade of Faith";
        //     } else if spell::is_status_expiring(job_details, "Blade of Truth Ready") {
        //         next_spell_name = "Blade of Truth";
        //     } else if spell::is_status_expiring(job_details, "Blade of Valor Ready") {
        //         next_spell_name = "Blade of Valor";
        //     } else if spell::is_status_expiring(job_details, "Goring Blade Ready") {
        //         next_spell_name = "Goring Blade";
        //     } else if aoe {
        //         if spell::is_status_expiring(job_details, "Divine Might") {
        //             next_spell_name = "Holy Spirit";
        //         } else if spell::is_status_expiring(job_details, "Prominence Ready") {
        //             next_spell_name = "Prominence";
        //         }
        //     } else {
        //         if spell::is_status_expiring(job_details, "Sepulchre Ready") {
        //             next_spell_name = "Sepulchre";
        //         } else if spell::is_status_expiring(job_details, "Supplication Ready") {
        //             next_spell_name = "Supplication";
        //         } else if spell::is_status_expiring(job_details, "Atonement Ready") {
        //             next_spell_name = "Atonement";
        //         } else if spell::is_status_expiring(job_details, "Royal Authority Ready") {
        //             next_spell_name = "Royal Authority";
        //         } else if spell::is_status_expiring(job_details, "Divine Might") {
        //             next_spell_name = "Holy Spirit";
        //         }
        //     }
        // }

        // check standart GCD
        if next_spell_name == "" {
            if spell::is_status_active_next_gcd(job_details, "Confiteor Ready") {
                next_spell_name = "Confiteor";
            } else if spell::is_status_active_next_gcd(job_details, "Blade of Faith Ready") {
                next_spell_name = "Blade of Faith";
            } else if spell::is_status_active_next_gcd(job_details, "Blade of Truth Ready") {
                next_spell_name = "Blade of Truth";
            } else if spell::is_status_active_next_gcd(job_details, "Blade of Valor Ready") {
                next_spell_name = "Blade of Valor";
            } else if spell::is_status_active_next_gcd(job_details, "Goring Blade Ready") {
                next_spell_name = "Goring Blade";
            // USe sepulchre combo asap if in buff, otherwise delay it
            } else if aoe {
                // Note on AOE : This will do a little mess if we missed a mob with Eclipse or Prominence
                // 3+ targets
                if spell::is_status_active_next_gcd(job_details, "Prominence Ready") {
                    if spell::is_status_active_next_gcd(job_details, "Divine Might") {
                        next_spell_name = "Holy Circle";
                    } else {
                        next_spell_name = "Prominence";
                    }
                } else {
                    next_spell_name = "Total Eclipse";
                }
            } else {
                // Single Target
                if spell::is_status_active_next_gcd(job_details, "Sepulchre Ready")
                    && spell::is_status_active_next_gcd(job_details, "Fight or Flight Buff")
                {
                    next_spell_name = "Sepulchre";
                } else if spell::is_status_active_next_gcd(job_details, "Supplication Ready")
                    && spell::is_status_active_next_gcd(job_details, "Fight or Flight Buff")
                {
                    next_spell_name = "Supplication";
                } else if spell::is_status_active_next_gcd(job_details, "Atonement Ready")
                    && spell::is_status_active_next_gcd(job_details, "Fight or Flight Buff")
                {
                    next_spell_name = "Atonement";
                } else if spell::is_status_active_next_gcd(job_details, "Royal Authority Ready") {
                    // Special case: Holy spirit only if it will disappear by royal authority
                    // Ortherwise it is kept as long as possible for mobility uptime
                    // Same, keep Sepulchre as long as possible to maxmize strong gcd in burst
                    if spell::is_status_active_next_gcd(job_details, "Sepulchre Ready") {
                        next_spell_name = "Sepulchre";
                    } else if spell::is_status_active_next_gcd(job_details, "Supplication Ready") {
                        next_spell_name = "Supplication";
                    } else if spell::is_status_active_next_gcd(job_details, "Atonement Ready") {
                        next_spell_name = "Atonement";
                    } else if spell::is_status_active_next_gcd(job_details, "Divine Might") {
                        next_spell_name = "Holy Spirit";
                    } else {
                        next_spell_name = "Royal Authority";
                    }
                } else if spell::is_status_active_next_gcd(job_details, "Riot Blade Ready") {
                    next_spell_name = "Riot Blade";
                } else {
                    next_spell_name = "Fast Blade";
                }
            }
        }
        if next_spell_name != "" {
            memory::communicate_next_spellcast(job_details, next_spell_name, aoe, alternate);
        }
    } else {
        error!("JOB not found in spell book");
    }
}

pub fn apply_spell_additional_effects(spell_name: &str, job_details: &mut JobDetails) {
    match spell_name {
        "Fight or Flight" => {
            job_details.burst_allowed = true;
            job_details.dump_resources = false;
        }
        "Imperator" => {
            job_details.burst_allowed = true;
            job_details.dump_resources = false;
        }

        _ => {}
    }
}
