// rotation_scholar.rs
// Rotation logic for scholar job

use log::error;

use crate::memory;
use crate::spell;
use crate::spell::is_spell_available_in_x_gcd;
use crate::types::{CharacterDetails, JobDetails};

pub fn update_next_spellcast(character_details: &mut CharacterDetails, aoe: bool, alternate: bool) {
    if let Some(job_details) = character_details.get_mut("SCH") {
        const MAX_WEAVE_NUMBER: i32 = 1;
        // update status

        // update stack min / max
        // Stack a = aetherflow
        if job_details.stacks_a > 3 {
            job_details.stacks_a = 3;
        } else if job_details.stacks_a < 0 {
            job_details.stacks_a = 0;
        }

        // check oGCD
        let mut next_spell_name = "";

        if (job_details.weaves < MAX_WEAVE_NUMBER) && spell::ogcd_end_before_next_gcd(job_details) {
            // --- A Priority: no drifting raid buffs
            if spell::is_spell_available_now(job_details, "Chain Stratagem") {
                next_spell_name = "Chain Stratagem";
            // Impact imminient is delayed because doing aetherflow is a priority
            } else if spell::is_status_active_now(job_details, "Impact Imminent")
                && !spell::is_spell_available_now(job_details, "Aetherflow")
            {
                next_spell_name = "Baneful Impaction";
            } else if should_aetherflow(job_details, aoe, alternate) {
                // Refresh charges with Aetherflow
                next_spell_name = "Aetherflow";
            } else if should_dump_aether_charges(job_details, aoe) {
                // Drain dump before using Aethercharge
                next_spell_name = "Energy Drain";
            }
        }

        // No oGCD selected, prepare next GCD

        // no check on EXPIRING GCD because not an easy occurance for this job

        // check normal GCD
        if next_spell_name == "" {
            if aoe {
                next_spell_name = "Art of War"
            } else {
                // reset dot end of burst
                if should_reset_dot_end_of_burst(job_details) {
                    next_spell_name = "Biolysis";
                // Reset Dot on time
                } else if spell::is_spell_available_next_gcd(job_details, "Biolysis") {
                    next_spell_name = "Biolysis";
                // Default GCD
                } else {
                    if alternate {
                        // Alternate : movement phase - too busy for precise weaving.
                        next_spell_name = "Ruin II"
                    } else {
                        next_spell_name = "Broil IV";
                    }
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

fn should_dump_aether_charges(job_details: &mut JobDetails, aoe: bool) -> bool {
    // If there is 1 / 2 /3 GCD left, and 1 / 2 /3 charges left, we should use them
    if !aoe && job_details.stacks_a > 0 {
        if (job_details.stacks_a >= 3) && is_spell_available_in_x_gcd(job_details, "Aetherflow", 3)
        {
            return true;
        }
        if (job_details.stacks_a >= 2) && is_spell_available_in_x_gcd(job_details, "Aetherflow", 2)
        {
            return true;
        }
        if (job_details.stacks_a >= 1) && is_spell_available_in_x_gcd(job_details, "Aetherflow", 2)
        {
            return true;
        }
    }
    return false;
}

fn should_aetherflow(job_details: &mut JobDetails, aoe: bool, alternate: bool) -> bool {
    if spell::is_spell_available_now(job_details, "Aetherflow") {
        // Mono target static, in burst : delay the cast to after dumping the remaining energy drains
        // In the other cases, it is usually better not to delay its cast
        if aoe || alternate {
            return true;
        } else {
            // Not in burst - Don't wait
            // In burst - wait for dumping the remanent charges
            // Enable automatic gestion of dissipation opening
            let in_burst_in_2_gcd = spell::is_status_still_active_in_two_gcd(job_details, "Burst");
            return job_details.stacks_a == 0 || !in_burst_in_2_gcd;
        }

        // FIXME - delay use only if there is a burst
    }
    return false;
}

// Opti suggested by guide, but makes the wound slide
fn should_reset_dot_end_of_burst(job_details: &mut JobDetails) -> bool {
    let dot_half_spent = !spell::is_status_active_in(job_details, "Dot", 15000); // 15s left on dot or less
    let in_burst_next_gcd = spell::is_status_active_next_gcd(job_details, "Burst");
    let in_burst_in_2_gcd = spell::is_status_still_active_in_two_gcd(job_details, "Burst");
    let end_of_burst_situation = in_burst_next_gcd && !in_burst_in_2_gcd;

    return end_of_burst_situation && dot_half_spent;
}

pub fn apply_spell_additional_effects(_spell_name: &str, _job_details: &mut JobDetails) {}

pub fn update_job_gauge(job_details: &mut JobDetails, data: &str) {
    let chunks: Vec<String> = data
        .as_bytes() // Convert the string to a byte slice
        .chunks(2) // Split into chunks of 2 bytes
        .map(|chunk| String::from_utf8_lossy(chunk).to_string()) // Convert each chunk back to a String
        .collect();

    let aether_charges: i32 =
        i32::from_str_radix(&chunks[1], 16).expect("Failed to convert chunk2");

    job_details.stacks_a = aether_charges;

    let _faery_gauge: i32 = i32::from_str_radix(&chunks[2], 16).expect("Failed to convert chunk3");
}
