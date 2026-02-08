// rotation_monk.rs
// Rotation logic for Monk job

use log::{debug, error};

use crate::memory;
use crate::monk_nadi::{self, Animal};
use crate::spell;
use crate::types::{CharacterDetails, JobDetails, SpellcastType};

pub fn update_next_spellcast(
    character_details: &mut CharacterDetails,
    spell_cast_type: SpellcastType,
    aoe_old: bool,
    alternate_old: bool,
) {
    if let Some(job_details) = character_details.get_mut("MNK") {
        const MAX_WEAVE_NUMBER: i32 = 1;
        // update status
        // check oGCD
        let mut next_spell_name = "";
        // let burst_allowed = spell::burst_allowed(job_details);
        // let delayed_burst = !burst_allowed;

        // Melee arrangement
        // 1 : normal
        // 2 : AOE
        // 3 : Only dist instant Mono
        // TBC : In serious play, Only Dist may be more useful than AOE.

        let mut distance_instant_cast_only = false;
        let mut aoe = false;

        match spell_cast_type {
            SpellcastType::First => (),
            SpellcastType::Second => aoe = true,
            SpellcastType::Third => distance_instant_cast_only = true,
            SpellcastType::Fourth => (),
        }

        // Check if we went out of opener (burst skill unavailable, burst buff over)
        if job_details.opener {
            if !spell::is_status_active_now(job_details, "Brother Buff")
                && !spell::is_spell_available_now(job_details, "Brotherhood")
            {
                job_details.opener = false
            }
        }

        // update stack min / max

        // Stack A : Coeurl's fury charge
        // if job_details.stacks_a > 2 {
        //     job_details.stacks_a = 2;
        // } else if job_details.stacks_a < 0 {
        //     job_details.stacks_a = 0;
        // }

        // Special treatment of distant cast
        if (next_spell_name == "") && distance_instant_cast_only {
            // oGCD
            if spell::ogcd_end_before_next_gcd(job_details) && should_dump_chakra(job_details) {
                next_spell_name = "Enlightenment";
            }
            // This is not the idal usage for those skills but better to use them is distance forced
            else if spell::is_status_active_next_gcd(job_details, "Fire's Rumination") {
                next_spell_name = "Fire's Reply";
            } else if spell::is_status_active_now(job_details, "Wind's Rumination") {
                next_spell_name = "Wind's Reply";
            } else if should_dump_chakra(job_details) {
                next_spell_name = "Enlightenment";
            } else {
                next_spell_name = "Forbidden Meditation";
            }
        }

        // High priority oGCDz

        // No clipping? - oGCD

        if (next_spell_name == "")
            && (job_details.weaves < MAX_WEAVE_NUMBER)
            && spell::ogcd_end_before_next_gcd(job_details)
        {
            // 120 m Burst
            if should_brotherhood(job_details) {
                next_spell_name = "Brotherhood"
            } else {
                // 1m burst
                // Note - it could also be delayed with the trigger...
                if spell::is_spell_available_now(job_details, "Riddle of Fire")
                    && !spell::spell_will_be_available_soon(job_details, "Brotherhood", 30000)
                {
                    next_spell_name = "Riddle of Fire"
                }
                // Chacka spending
                else if should_dump_chakra(job_details) {
                    if aoe {
                        next_spell_name = "Enlightenment"
                    } else {
                        next_spell_name = "The Forbidden Chakra"
                    }
                }
                // 90 s bonus
                else if spell::is_spell_available_now(job_details, "Riddle of Wind")
                    && !spell::spell_will_be_available_soon(job_details, "Brotherhood", 15000)
                {
                    next_spell_name = "Riddle of Wind"
                } else
                // Perfect balance check is last. It is very possibe to bug out (death, accidental use of charge)
                //  so it should be last not to block out anything in case
                if should_perfect_balance(job_details) {
                    next_spell_name = "Perfect Balance";
                }
            }
        }

        // No oGCD selected, prepare next GCD

        // check expiring GCD

        // Expiring Buff follow ups
        if next_spell_name == "" {
            let expiring_fire = spell::is_status_active_next_gcd(job_details, "Fire's Rumination")
                && !spell::is_status_still_active_in_two_gcd(job_details, "Fire's Rumination");
            let expiring_wind = spell::is_status_active_next_gcd(job_details, "Wind's Rumination")
                && !spell::is_status_still_active_in_two_gcd(job_details, "Wind's Rumination");
            // if expiring_finisher {
            //     next_spell_name = "Masterful Blitz";
            // } else
            if expiring_fire {
                next_spell_name = "Fire's Reply";
            } else if expiring_wind {
                next_spell_name = "Wind's Reply";
            }
        }

        // Priority over the buff follow up to use fire's formless fist
        // Formless fist = opportunity to get one more opo
        if next_spell_name == "" && spell::is_status_active_next_gcd(job_details, "Formless Fist") {
            next_spell_name = get_next_opoopo_spell(job_details, aoe);
        }

        // Buff follow ups
        if next_spell_name == "" {
            let is_after_opo = spell::is_status_active_next_gcd(job_details, "Raptor Form");
            let bortherhood_soon =
                spell::spell_will_be_available_soon(job_details, "Brotherhood", 16000);
            let fire_soon =
                spell::spell_will_be_available_soon(job_details, "Riddle of Fire", 5000);

            let in_balance = spell::is_status_active_now(job_details, "Balance Buff");
            let finisher_time = 3 <= monk_nadi::get_nb_beast_chakra(&job_details) && in_balance;

            // Perfect balance finisher
            if finisher_time && !bortherhood_soon && !fire_soon {
                next_spell_name = "Masterful Blitz";
            } else if is_after_opo
                && spell::is_status_active_next_gcd(job_details, "Fire's Rumination")
                && !in_balance
                && !finisher_time
            {
                next_spell_name = "Fire's Reply";
            } else if spell::is_status_active_now(job_details, "Wind's Rumination")
                && !in_balance
                && !finisher_time
            {
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
            memory::communicate_next_spellcast(job_details, spell_cast_type, next_spell_name);
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

pub fn apply_spell_additional_effects(spell_name: &str, _job_details: &mut JobDetails) {
    match spell_name {
        _ => {}
    }
}

pub fn update_job_gauge(job_details: &mut JobDetails, data: &str) {
    // debug!("Job gauge update: {}", data);
    let chunks: Vec<String> = data
        .as_bytes() // Convert the string to a byte slice
        .chunks(2) // Split into chunks of 2 bytes
        .map(|chunk| String::from_utf8_lossy(chunk).to_string()) // Convert each chunk back to a String
        .collect();

    // let chunk1: i32 = i32::from_str_radix(&chunks[0], 16).expect("Failed to convert chunk1");
    // Chunck 2 = Chackra
    let chunk2: i32 = i32::from_str_radix(&chunks[2], 16).expect("Failed to convert chunk2");
    job_details.stacks_b = chunk2;

    let chunk3: i32 = i32::from_str_radix(&chunks[3], 16).expect("Failed to convert chunk3");
    let beast_chackra_1 = Animal::from_i32(chunk3);
    // monk_nadi::set_animal_slot_1(job_details, beast_chackra_1);

    let chunk4: i32 = i32::from_str_radix(&chunks[4], 16).expect("Failed to convert chunk4");
    let beast_chackra_2 = Animal::from_i32(chunk4);
    // monk_nadi::set_animal_slot_2(job_details, beast_chackra_2);

    let chunk5: i32 = i32::from_str_radix(&chunks[5], 16).expect("Failed to convert chunk5");
    let beast_chackra_3 = Animal::from_i32(chunk5);
    // monk_nadi::set_animal_slot_3(job_details, beast_chackra_3);

    // // Chunck 6 = Beast fury
    // let chunk6: i32 = i32::from_str_radix(&chunks[5], 16).expect("Failed to convert chunk6");

    // Chunk 7 : nadi
    let chunk7: i32 = i32::from_str_radix(&chunks[7], 16).expect("Failed to convert chunk7");
    let lunar_nadi = (chunk7 & 1) != 0;
    let solar_nadi = ((chunk7 >> 1) & 1) != 0;

    let chunk8: i32 = i32::from_str_radix(&chunks[7], 16).expect("Failed to convert chunk8");
    let chunk9: i32 = i32::from_str_radix(&chunks[8], 16).expect("Failed to convert chunk9");
    let chunk10: i32 = i32::from_str_radix(&chunks[9], 16).expect("Failed to convert chunk10");
    let chunk11: i32 = i32::from_str_radix(&chunks[10], 16).expect("Failed to convert chunk11");
    let chunk12: i32 = i32::from_str_radix(&chunks[11], 16).expect("Failed to convert chunk12");
    let chunk13: i32 = i32::from_str_radix(&chunks[12], 16).expect("Failed to convert chunk13");
    let chunk14: i32 = i32::from_str_radix(&chunks[13], 16).expect("Failed to convert chunk14");
    let chunk15: i32 = i32::from_str_radix(&chunks[14], 16).expect("Failed to convert chunk15");
    let chunk16: i32 = i32::from_str_radix(&chunks[15], 16).expect("Failed to convert chunk16");

    monk_nadi::set_nadi_state(
        job_details,
        [beast_chackra_1, beast_chackra_2, beast_chackra_3],
        lunar_nadi,
        solar_nadi,
    );

    // Debug output for each chunk
    // debug!("chunk1 = {}", chunk1);
    debug!("chunk2 = {}", chunk2);
    debug!("chunk3 = {}", chunk3);
    debug!("chunk4 = {}", chunk4);
    debug!("chunk5 = {}", chunk5);
    // debug!("chunk6 = {}", chunk6);
    debug!("chunk7 = {}", chunk7);
    debug!("chunk8 = {}", chunk8);
    debug!("chunk9 = {}", chunk9);
    debug!("chunk10 = {}", chunk10);
    debug!("chunk11 = {}", chunk11);
    debug!("chunk12 = {}", chunk12);
    debug!("chunk13 = {}", chunk13);
    debug!("chunk14 = {}", chunk14);
    debug!("chunk15 = {}", chunk15);
    debug!("chunk16 = {}", chunk16);
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

// FIXME forced pot usage in opener

fn should_perfect_balance(job_details: &mut JobDetails) -> bool {
    // If we already chained 2 PB, don't do a third
    // This is also tested further but testing here avoid some possible bug
    if spell::is_status_active_now(job_details, "In Second Balance") {
        return false;
    }

    // Fire used recently, Brotherhood in a while -> odd window
    let in_odd_window = spell::is_status_active_now(job_details, "Fire Buff")
        && !spell::is_status_active_now(job_details, "Brother Buff")
        && !spell::spell_will_be_available_soon(job_details, "Brotherhood", 35000);

    let odd_window_lead_time = 7000;
    let in_lead_time_of_odd_window =
        spell::spell_will_be_available_soon(job_details, "Riddle of Fire", odd_window_lead_time)
            && !spell::is_spell_available_in_x_gcd(job_details, "Brotherhood", 20);
    // Set the right amount of GCD

    // "2 to 4 GCD before RoF and Brotherhood"
    // We want the finisher inside the buff - So 3 full GCD max - 7s
    let even_window_lead_time = 7000;
    let in_lead_time_of_even_window =
        spell::spell_will_be_available_soon(job_details, "Brotherhood", even_window_lead_time);

    // Emergency time : brotherhood available or will available before we can prepare a PB. Start the first PB immediately.
    let in_emegency_time_of_even_window = !job_details.opener
        && spell::spell_will_be_available_soon(job_details, "Brotherhood", 2500);

    let lunar_nadi_next = monk_nadi::do_we_want_lunar_nadi(job_details);
    let solar_nadi_next = !lunar_nadi_next;

    let finisher_time = 3 <= monk_nadi::get_nb_beast_chakra(&job_details);
    let in_perfect_balance = spell::is_status_active_now(job_details, "Balance Buff");
    // let in_extended_perfect_balance =
    //     spell::is_status_active_now(job_details, "Balance Buff") || finisher_time;

    if in_perfect_balance {
        return false;
    }

    // Used to ensure that we don't throw too many balance in one window
    let already_did_a_perfect_balance = spell::is_status_active_now(job_details, "Recent Balance");

    // FIXME - case of a delayed burst. Need to do the 2m peferct balance ahead still
    // FIXME - there is a design decision to do, because normaly we don't want to delay brotherhood.
    // Because of the "trigger it on an oppo" the PB is not always aligned anyway.
    // Maybe we should trigger PB anyway if we are 80 s from last one -> means we will cap the charges

    // !! Only One charge per odd window -> we use a buff to track it in case

    // let fire_riddle_soon_or_now = spell_will_be_available_soon(job_details, "spell_name", max_waiting_duration)

    // Perfect balance Solar Odd window "2 to 7s before fire trigger" -> 1 full gcd to 3,5 full gcd before fire
    // We want to activate it after an opo, before the fire, and be sure that the finisher GCD is after the fire pro
    // in_lead_time_of_solar_odd_window // include solar and odd windows test
    let should_start_solar_odd_window_perfect_balance = solar_nadi_next
        && in_lead_time_of_odd_window
        && spell::is_status_active_now(job_details, "Raptor Form")
        && !already_did_a_perfect_balance;

    // Perfect balance Lunar / Phantom Odd window AFTER fire trigger"
    // Brother not coming (50s +)
    // Fire is active
    // Check it is the first in that odd window
    // Note: in case of failed pre balance for solar, tihs should catch it in th odd burst window !
    let should_start_lunar_odd_window_perfect_balance = lunar_nadi_next && in_odd_window // include fire has been pressed
        && !already_did_a_perfect_balance;

    // Even window
    // When you press an Opo-opo GCD 2-4 GCDs before RoF and BH come off cooldown, follow it with a Perfect Balance
    // Exact GCD to be checked again
    // If Raptor and soon a brotherhood, AND BURST IS OK, pre even PB
    // Note that the burst oGCD are used on CD anyway
    // If brotherhood is coming (exact time tbc) and on raptor form -> PB
    let should_start_even_window_first_perfect_balance = spell::burst_allowed(job_details)
        && in_lead_time_of_even_window
        && spell::is_status_active_now(job_details, "Raptor Form");

    // Warning - to be tested - failed to start before BH is coming, we should start PB ASAP
    let emergency_even_window_first_perfect_balance = spell::burst_allowed(job_details)
        && in_emegency_time_of_even_window
        && !already_did_a_perfect_balance;

    // Second perfect balance in 2m burst
    // If in brotherhood, not in balance, not in finisher, not in free form, not waiting fire reply, activateit
    let should_start_even_window_second_perfect_balance =
        spell::is_status_active_now(job_details, "Brother Buff")
            && !spell::is_status_active_now(job_details, "In Second Balance")
            && !spell::is_status_active_now(job_details, "Balance Buff")
            && !finisher_time
            && !spell::is_status_active_now(job_details, "Fire's Rumination")
            && !spell::is_status_active_now(job_details, "Formless Fist")
            && !spell::is_status_active_now(job_details, "Wind's Rumination")
            && spell::is_status_active_in(job_details, "Brother Buff", 6000); // avoid 3rd PB

    // default don't activate Perfect Balance
    return should_start_even_window_first_perfect_balance
        || should_start_even_window_second_perfect_balance
        || emergency_even_window_first_perfect_balance
        || should_start_lunar_odd_window_perfect_balance
        || should_start_solar_odd_window_perfect_balance;
}

fn should_dump_chakra(job_details: &mut JobDetails) -> bool {
    if job_details.stacks_b >= 5 {
        if job_details.opener {
            // In opener, the first usage is delayed to after the Buffing
            return !spell::spell_will_be_available_soon(job_details, "Riddle of Fire", 5000);
        } else {
            return true;
        }
    }
    return false;
}

// Brothehood is the main burst CD - it is the one delayed with burst_allowed to false
// In opener, we should wait 5s (2,5 gcd) befor eits usage, for buff alignement
// "DK 5" opener
fn should_brotherhood(job_details: &mut JobDetails) -> bool {
    if spell::burst_allowed(job_details) {
        // Fire alignment checks are supposed to reduce drift between bortherhood and riddle of fire.
        let in_fire = spell::is_status_active_now(job_details, "Fire Buff");
        let fire_ready = spell::spell_will_be_available_soon(job_details, "Riddle of Fire", 750)
            && spell::is_spell_available_next_gcd(job_details, "Riddle of Fire");
        let fire_alignement_ok = in_fire || fire_ready;

        if spell::is_spell_available_now(job_details, "Brotherhood") && fire_alignement_ok {
            // Nadi means we finished a PB - no delay - here in case of bug
            let has_nadi =
                monk_nadi::has_solar_nadi(job_details) || monk_nadi::has_lunar_nadi(job_details);

            // Third GCD is PB 1 charge left
            let third_gcd = get_perfect_balance_charges(job_details) == 1;

            let should_delay_burst_for_opener = job_details.opener && !third_gcd && !has_nadi;

            return !should_delay_burst_for_opener;
        }
    }
    return false;
}

pub fn get_perfect_balance_charges(job_details: &mut JobDetails) -> u8 {
    if spell::is_status_active_now(job_details, "Balance Buff") {
        let nb_active_beast_chakra = monk_nadi::get_nb_beast_chakra(job_details);
        return 3 - nb_active_beast_chakra;
    }
    return 0;
}
