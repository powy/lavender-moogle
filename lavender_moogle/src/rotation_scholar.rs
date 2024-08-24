// rotation_scholar.rs
// Rotation logic for scholar job

use log::error;

use crate::types::{CharacterDetails, JobDetails};
use crate::spell;
use crate::memory;


pub fn update_next_spellcast(character_details: &mut CharacterDetails) {
    if let Some(job_details) = character_details.get_mut("SCH") {
        // update status

        // update stack min / max
        // Stack a = aetherflow
        if job_details.stacks_a > 2 {
            job_details.stacks_a = 2;
        } else if job_details.stacks_a < 0 {
            job_details.stacks_a = 0;
        }

        // check oGCD
        let mut next_spell_name = "";

        if spell::ogcd_end_before_next_gcd(job_details)
        {
            // --- A Priority: no drifting raid buffs
            if spell::is_spell_available_now(job_details, "Chain Stratagem") {
                next_spell_name = "Chain Stratagem";
            } else if spell::is_status_active_now(job_details, "Impact imminent") {
                next_spell_name = "Baneful Impaction";
            }
        }

        // No oGCD selected, prepare next GCD            

        // no check on EXPIRING GCD because not an easy occurance for this job

        // check normal GCD
        if next_spell_name == "" {

            // big summons
            if spell::is_spell_available_next_gcd(job_details, "Biolysis") {
                next_spell_name = "Biolysis";
            // lesser summons
            } else {
                next_spell_name = "Broil IV";
            }
        }
        if next_spell_name != "" {
            memory::communicate_next_spellcast(job_details, next_spell_name);
        }
    } else {
        error!("JOB not found in spell book");
    }
}

pub fn apply_spell_additional_effects(_spell_name: &str, _job_details: &mut JobDetails) {

}
