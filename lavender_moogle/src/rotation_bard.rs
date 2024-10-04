// rotation_bard.rs
// Rotation logic for bard job

use log::error;

use crate::memory;
use crate::spell;
use crate::types::{CharacterDetails, JobDetails};

pub fn update_next_spellcast(character_details: &mut CharacterDetails) {
    if let Some(job_details) = character_details.get_mut("BRD") {
        // update status
        // check oGCD
        let mut next_spell_name = "";

        // update stack min / max

        // if job_details.stacks_a > 4 {
        //     job_details.stacks_a = 4;
        // } else if job_details.stacks_a < 0 {
        //     job_details.stacks_a = 0;
        // }

        // High priority oGCD

        // No clipping? - oGCD

        // if next_spell_name == "" && spell::ogcd_end_before_next_gcd(job_details) {
        //     // BURST oGCD
        //     // FIXME

        //     // out of burst oGCD
        //     if next_spell_name == "" {
        //         // Fix Overcap on Bloodletter
        //         // FIXME
        //         //proc
        //         // Sidewinder on CD
        //         if spell::is_spell_available_now(job_details, "Sidewinder") {
        //             next_spell_name = "Sidewinder";
        //         }
        //         // Empyreal Arrow on CD
        //         else if spell::is_spell_available_now(job_details, "Empyreal Arrow") {
        //             next_spell_name = "Empyreal Arrow";
        //         }
        //         // Avoid bloodletter overcap
        //         // Fixme
        //     }
        // }
        // No oGCD selected, prepare next GCD

        // No dot - do dot - emergency dot
        // No dot - do dot - emergency dot
        if next_spell_name == "" {
            if !spell::is_status_active_next_gcd(job_details, "Storm DoT") {
                next_spell_name = "Stormbite";
            } else if !spell::is_status_active_next_gcd(job_details, "Caustic DoT") {
                next_spell_name = "Caustic Bite";
            } else if !spell::is_status_still_active_in_two_gcd(job_details, "Storm DoT")
                || !spell::is_status_still_active_in_two_gcd(job_details, "Caustic DoT")
            {
                next_spell_name = "Iron Jaws";
            }
        }

        // check standart GCD
        // Not in burst window - avoid overcap blood or classic combo
        if next_spell_name == "" {
            if spell::is_status_active_next_gcd(job_details, "Hawk's Eye") {
                next_spell_name = "Refulgent Arrow";
            } else {
                next_spell_name = "Burst Shot";
            }
        }
        if next_spell_name != "" {
            memory::communicate_next_spellcast(job_details, next_spell_name);
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

pub fn update_job_gauge(_job_details: &mut JobDetails, _data: &str) {
    // Placeholder for gauge updates
}

// let chunk1: u8 = u8::from_str_radix(&chunks[0], 16).expect("Failed to convert chunk1");
// let chunk2: u8 = u8::from_str_radix(&chunks[1], 16).expect("Failed to convert chunk2");
// let chunk3: u8 = u8::from_str_radix(&chunks[2], 16).expect("Failed to convert chunk3");
// let chunk4: u8 = u8::from_str_radix(&chunks[3], 16).expect("Failed to convert chunk4");
// let chunk5: u8 = u8::from_str_radix(&chunks[4], 16).expect("Failed to convert chunk5");
// let chunk6: u8 = u8::from_str_radix(&chunks[5], 16).expect("Failed to convert chunk6");
// let chunk7: u8 = u8::from_str_radix(&chunks[6], 16).expect("Failed to convert chunk7");
// let chunk8: u8 = u8::from_str_radix(&chunks[7], 16).expect("Failed to convert chunk8");
// let chunk9: u8 = u8::from_str_radix(&chunks[8], 16).expect("Failed to convert chunk9");
// let chunk10: u8 = u8::from_str_radix(&chunks[9], 16).expect("Failed to convert chunk10");
// let chunk11: u8 = u8::from_str_radix(&chunks[10], 16).expect("Failed to convert chunk11");
// let chunk12: u8 = u8::from_str_radix(&chunks[11], 16).expect("Failed to convert chunk12");
// let chunk13: u8 = u8::from_str_radix(&chunks[12], 16).expect("Failed to convert chunk13");
// let chunk14: u8 = u8::from_str_radix(&chunks[13], 16).expect("Failed to convert chunk14");
// let chunk15: u8 = u8::from_str_radix(&chunks[14], 16).expect("Failed to convert chunk15");
// let chunk16: u8 = u8::from_str_radix(&chunks[15], 16).expect("Failed to convert chunk16");
