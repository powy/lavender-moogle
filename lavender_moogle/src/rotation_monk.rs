// rotation_monk.rs
// Rotation logic for Monk job

use log::{debug, error};

use crate::memory;
use crate::monk_nadi;
use crate::spell;
use crate::types::{CharacterDetails, JobDetails};

pub fn update_next_spellcast(character_details: &mut CharacterDetails, aoe: bool, alternate: bool) {
    if let Some(job_details) = character_details.get_mut("MNK") {
        const MAX_WEAVE_NUMBER: i32 = 2;
        // update status
        // check oGCD
        let mut next_spell_name = "";

        // update stack min / max

        // Stack A : Coeurl's fury charge
        if job_details.stacks_a > 2 {
            job_details.stacks_a = 2;
        } else if job_details.stacks_a < 0 {
            job_details.stacks_a = 0;
        }

        // High priority oGCD

        // No clipping? - oGCD

        if (next_spell_name == "")
            && (job_details.weaves < MAX_WEAVE_NUMBER)
            && spell::ogcd_end_before_next_gcd(job_details)
        {
            // 120 m Burst
            if !alternate && spell::is_spell_available_now(job_details, "Brotherhood") {
                next_spell_name = "Brotherhood"
            } else {
                // 1m burst
                let bortherhood_soon =
                    spell::spell_will_be_available_soon(job_details, "Brotherhood", 30000);
                if !bortherhood_soon && spell::is_spell_available_now(job_details, "Riddle of Fire")
                {
                    next_spell_name = "Riddle of Fire"
                }
                // Chacka spending
                else if job_details.stacks_b >= 5 {
                    if aoe {
                        next_spell_name = "Enlightenment"
                    } else {
                        next_spell_name = "The Forbidden Chakra"
                    }
                }
                // 90 s bonus
                else if spell::is_spell_available_now(job_details, "Riddle of Wind") {
                    next_spell_name = "Riddle of Wind"
                }
            }
        }

        // No oGCD selected, prepare next GCD

        // check expiring GCD

        // Expiring Buff follow ups
        if next_spell_name == "" {
            let expiring_fire = spell::is_status_active_now(job_details, "Fire's Rumination")
                && !spell::is_status_still_active_in_two_gcd(job_details, "Fire's Rumination");
            let expiring_wind = spell::is_status_active_now(job_details, "Wind's Rumination")
                && !spell::is_status_still_active_in_two_gcd(job_details, "Wind's Rumination");
            if expiring_fire {
                next_spell_name = "Fire's Reply";
            } else if expiring_wind {
                next_spell_name = "Wind's Reply";
            }
        }

        // check 2m burst GCD
        // check 1m burst GCD
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

        // Perfect balance finisher
        // TODO : Improve by checking if we should finish in a burst window...
        if next_spell_name == "" && spell::is_status_active_now(job_details, "Finisher Buff") {
            next_spell_name = "Masterful Blitz";
        }

        // Buff follow ups
        if next_spell_name == "" {
            let well_timed_fire = spell::is_status_active_now(job_details, "Fire's Rumination")
                && spell::is_status_active_now(job_details, "Raptor Form");

            if well_timed_fire {
                next_spell_name = "Fire's Reply";
            } else if spell::is_status_active_now(job_details, "Wind's Rumination") {
                next_spell_name = "Wind's Reply";
                // TODO : improve by delaying wind if we need to align fire on Opo ?
            }
        }

        // Perfect balance GCDs
        if next_spell_name == "" && spell::is_status_active_now(job_details, "Balance Buff") {
            next_spell_name = get_next_balance_gcd(job_details, aoe);
        }

        // check standart GCD
        // Not in burst window - avoid overcap blood or classic combo
        if next_spell_name == "" {
            if spell::is_status_active_next_gcd(job_details, "Raptor Form") {
                next_spell_name = get_next_raptor_spell(job_details, aoe);
            } else if spell::is_status_active_next_gcd(job_details, "Coeurl Form") {
                next_spell_name = get_next_coeurl_spell(job_details, aoe);
            } else {
                // Opo
                next_spell_name = get_next_opoopo_spell(job_details, aoe);
            }
        }
        if next_spell_name != "" {
            memory::communicate_next_spellcast(job_details, next_spell_name, aoe, alternate);
        }
    } else {
        error!("JOB not found in spell book");
    }
}

pub fn get_next_opoopo_spell(job_details: &mut JobDetails, aoe: bool) -> &'static str {
    if aoe {
        return "Shadow of the Destroyer";
    } else if spell::is_status_active_next_gcd(job_details, "Opo-opo's Fury") {
        return "Leaping Opo";
    } else {
        return "Dragon Kick";
    }
}

pub fn get_next_raptor_spell(job_details: &mut JobDetails, aoe: bool) -> &'static str {
    if aoe {
        return "Four-point Fury";
    } else if spell::is_status_active_next_gcd(job_details, "Raptor's Fury") {
        return "Rising Raptor";
    } else {
        return "Twin Snakes";
    }
}

pub fn get_next_coeurl_spell(job_details: &mut JobDetails, aoe: bool) -> &'static str {
    if aoe {
        return "Rockbreaker";
    } else if job_details.stacks_a > 0 {
        return "Pouncing Coeurl";
    } else {
        return "Demolish";
    }
}

pub fn apply_spell_additional_effects(spell_name: &str, job_details: &mut JobDetails) {
    match spell_name {
        "Perfect Balance" => monk_nadi::add_three_perfect_balance_charges(job_details),
        "Elixir Burst" => elixir_finish(job_details),
        "Rising Phoenix" => phoenix_finish(job_details),
        "Celestial Revolution" => celestial_finish(job_details),
        "Phantom Rush" => phantom_finish(job_details),
        _ => {}
    }

    if spell::is_status_active_now(job_details, "Balance Buff") {
        if matches!(
            spell_name,
            "Leaping Opo" | "Shadow of the Destroyer" | "Dragon Kick"
        ) {
            monk_nadi::add_opo_charge(job_details);
        }

        if matches!(
            spell_name,
            "Four-point Fury" | "Rising Raptor" | "Twin Snakes"
        ) {
            monk_nadi::add_raptor_charge(job_details);
        }

        if matches!(spell_name, "Rockbreaker" | "Pouncing Coeurl" | "Demolish") {
            monk_nadi::add_coeurl_charge(job_details);
        }
    }
}

// pub fn get_next_perfect_balance_gcd(_job_details: &mut JobDetails, _aoe: bool) {}

pub fn update_job_gauge(job_details: &mut JobDetails, data: &str) {
    // debug!("Job gauge update: {}", data);
    let chunks: Vec<String> = data
        .as_bytes() // Convert the string to a byte slice
        .chunks(2) // Split into chunks of 2 bytes
        .map(|chunk| String::from_utf8_lossy(chunk).to_string()) // Convert each chunk back to a String
        .collect();

    // let chunk1: i32 = i32::from_str_radix(&chunks[0], 16).expect("Failed to convert chunk1");
    // Chunck 2 = Chackra
    let chunk2: i32 = i32::from_str_radix(&chunks[1], 16).expect("Failed to convert chunk2");
    job_details.stacks_b = chunk2;
    // let chunk3: i32 = i32::from_str_radix(&chunks[2], 16).expect("Failed to convert chunk3");
    // let chunk4: i32 = i32::from_str_radix(&chunks[3], 16).expect("Failed to convert chunk4");
    // let chunk5: i32 = i32::from_str_radix(&chunks[4], 16).expect("Failed to convert chunk5");
    // // Chunck 6 = Beast fury
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

fn get_next_balance_gcd(job_details: &mut JobDetails, aoe: bool) -> &'static str {
    let next_form = monk_nadi::next_animal_in_balance_phase(job_details);
    match next_form {
        "Opo-opo" => return get_next_opoopo_spell(job_details, aoe),
        "Raptor" => return get_next_raptor_spell(job_details, aoe),
        "Coeurl" => return get_next_coeurl_spell(job_details, aoe),
        _ => {}
    }
    return get_next_opoopo_spell(job_details, aoe);
}

fn celestial_finish(job_details: &mut JobDetails) {
    if monk_nadi::has_lunar_nadi(job_details) {
        monk_nadi::activate_solar_nadi(job_details);
    } else {
        monk_nadi::activate_lunar_nadi(job_details);
    }
    monk_nadi::reset_animal_charges(job_details);
    spell::deactivate_status(job_details, "Finisher Buff");
}

fn elixir_finish(job_details: &mut JobDetails) {
    monk_nadi::activate_lunar_nadi(job_details);
    monk_nadi::reset_animal_charges(job_details);
    spell::deactivate_status(job_details, "Finisher Buff");
}

fn phoenix_finish(job_details: &mut JobDetails) {
    monk_nadi::activate_solar_nadi(job_details);
    monk_nadi::reset_animal_charges(job_details);
    spell::deactivate_status(job_details, "Finisher Buff");
}

fn phantom_finish(job_details: &mut JobDetails) {
    monk_nadi::reset_animal_charges(job_details);
    monk_nadi::reset_nadi(job_details);
    spell::deactivate_status(job_details, "Finisher Buff");
}
