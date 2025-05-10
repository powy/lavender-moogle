// rotation_.rs
// Rotation logic for  job

use log::{debug, error};

use crate::memory;
use crate::spell;
use crate::types::{CharacterDetails, JobDetails};

pub fn update_next_spellcast(character_details: &mut CharacterDetails, aoe: bool, alternate: bool) {
    if let Some(job_details) = character_details.get_mut("FIXME") {
        const MAX_WEAVE_NUMBER: i32 = 2;
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
        // if (job_details.weaves < MAX_WEAVE_NUMBER) && spell::ogcd_end_before_next_gcd(job_details) {
        // }

        // if next_spell_name == "" && spell::ogcd_end_before_next_gcd(job_details)
        // {

        // }

        // No oGCD selected, prepare next GCD

        // check expiring GCD
        // if next_spell_name == "" {
        // }

        // check burst GCD
        // if next_spell_name == ""
        //     && spell::is_status_active_next_gcd(job_details, "2m Burst")
        //     && spell::is_status_active_next_gcd(job_details, "1m Burst")
        // {
        //         if spell::is_status_active_next_gcd(job_details, "Scorn") {
        //             next_spell_name = "Disesteem";
        //         } else if job_details.secondary_bar >= 50 {
        //             next_spell_name = "Bloodspiller";
        //         }
        // }

        // check standart GCD
        // Not in burst window - avoid overcap blood or classic combo
        if next_spell_name == "" {
            if spell::is_status_active_next_gcd(job_details, "XXX Ready") {
                next_spell_name = "XXX";
            } else {
                next_spell_name = "YYY";
            }
        }
        if next_spell_name != "" {
            memory::communicate_next_spellcast(job_details, next_spell_name, aoe, alternate);
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
    // debug!("Job gauge update: {}", data);
    // let chunks: Vec<String> = data
    //     .as_bytes() // Convert the string to a byte slice
    //     .chunks(2) // Split into chunks of 2 bytes
    //     .map(|chunk| String::from_utf8_lossy(chunk).to_string()) // Convert each chunk back to a String
    //     .collect();

    // let chunk1: i32 = i32::from_str_radix(&chunks[0], 16).expect("Failed to convert chunk1");
    // let chunk2: i32 = i32::from_str_radix(&chunks[1], 16).expect("Failed to convert chunk2");
    // let chunk3: i32 = i32::from_str_radix(&chunks[2], 16).expect("Failed to convert chunk3");
    // let chunk4: i32 = i32::from_str_radix(&chunks[3], 16).expect("Failed to convert chunk4");
    // let chunk5: i32 = i32::from_str_radix(&chunks[4], 16).expect("Failed to convert chunk5");
    // let chunk6: i32 = i32::from_str_radix(&chunks[5], 16).expect("Failed to convert chunk6");
    // let chunk7: i32 = i32::from_str_radix(&chunks[6], 16).expect("Failed to convert chunk7");
    // let chunk8: i32 = i32::from_str_radix(&chunks[7], 16).expect("Failed to convert chunk8");
    // let chunk9: i32 = i32::from_str_radix(&chunks[8], 16).expect("Failed to convert chunk9");
    // let chunk10: i32 = i32::from_str_radix(&chunks[9], 16).expect("Failed to convert chunk10");
    // let chunk11: i32 = i32::from_str_radix(&chunks[10], 16).expect("Failed to convert chunk11");
    // let chunk12: i32 = i32::from_str_radix(&chunks[11], 16).expect("Failed to convert chunk12");
    // let chunk13: i32 = i32::from_str_radix(&chunks[12], 16).expect("Failed to convert chunk13");
    // let chunk14: i32 = i32::from_str_radix(&chunks[13], 16).expect("Failed to convert chunk14");
    // let chunk15: i32 = i32::from_str_radix(&chunks[14], 16).expect("Failed to convert chunk15");
    // let chunk16: i32 = i32::from_str_radix(&chunks[15], 16).expect("Failed to convert chunk16");

    // // Debug output for each chunk
    // debug!("chunk1 = {}", chunk1);
    // debug!("chunk2 = {}", chunk2);
    // debug!("chunk3 = {}", chunk3);
    // debug!("chunk4 = {}", chunk4);
    // debug!("chunk5 = {}", chunk5);
    // debug!("chunk6 = {}", chunk6);
    // debug!("chunk7 = {}", chunk7);
    // debug!("chunk8 = {}", chunk8);
    // debug!("chunk9 = {}", chunk9);
    // debug!("chunk10 = {}", chunk10);
    // debug!("chunk11 = {}", chunk11);
    // debug!("chunk12 = {}", chunk12);
    // debug!("chunk13 = {}", chunk13);
    // debug!("chunk14 = {}", chunk14);
    // debug!("chunk15 = {}", chunk15);
    // debug!("chunk16 = {}", chunk16);
}
