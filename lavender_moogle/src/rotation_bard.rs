// rotation_bard.rs
// Rotation logic for bard job

use log::{debug, error};

use crate::memory;
use crate::spell;
use crate::types::{CharacterDetails, JobDetails, SpellcastType};

pub fn update_next_spellcast(
    character_details: &mut CharacterDetails,
    spell_cast_type: SpellcastType,
    _aoe: bool,
    _alternate: bool,
) {
    if let Some(job_details) = character_details.get_mut("BRD") {
        let max_weave_number: i32 = 2;

        // update status
        // check oGCD
        let mut next_spell_name = "";

        let mut aoe = false;
        let mut no_dot_rotation = false;

        // VARIATIONS Physical Ranged
        // 1 : normal
        // 2 : AOE
        // TBC 3 and 4 for Dot management ?

        match spell_cast_type {
            SpellcastType::First => (),
            SpellcastType::Second => aoe = true,
            SpellcastType::Third => no_dot_rotation = true,
            SpellcastType::Fourth => (),
        }

        // The whole file could be dsimplified by passing down this bool instead of recalculating it 10 times
        let dot_allowed = are_dot_allowed(job_details, aoe, no_dot_rotation);
        let iron_jaw_castable = is_iron_jaws_castable(job_details, aoe, no_dot_rotation);

        // Hack to detect the very first cast of the opener and do the HB shot
        // We should do a GCD (no oGCD) and wanderer not started
        if job_details.opener && are_dot_allowed(job_details, aoe, no_dot_rotation) {
            if job_details.stacks_b == 0 {
                next_spell_name = "Heartbreak Shot";
            } else if !spell::is_status_active_next_gcd(job_details, "Storm DoT") {
                next_spell_name = "Stormbite";
            } else if spell::is_spell_available_now(job_details, "the Wanderer's Minuet") {
                next_spell_name = "the Wanderer's Minuet";
            } else if spell::is_spell_available_now(job_details, "Empyreal Arrow") {
                next_spell_name = "Empyreal Arrow";
            } else if !spell::is_status_active_next_gcd(job_details, "Caustic DoT") {
                next_spell_name = "Caustic Bite";
            } else if spell::is_spell_available_now(job_details, "Battle Voice")
                && spell::in_late_ogcd_timing(job_details)
            {
                next_spell_name = "Battle Voice";
            } else {
                next_spell_name = "Burst Shot";
            }
        }

        // Common logic from all classes
        let normal_ogcd_allowed_conditions =
            (job_details.weaves < max_weave_number) && spell::ogcd_end_before_next_gcd(job_details);

        // Special case for BRD : we have higher attack speed in the Paeon Army, making a mess of what is castable. To prevent problem, just skip late weave spot
        let ogcd_restricted_because_paeon = spell::is_status_active_now(job_details, "Army song")
            && (spell::in_late_ogcd_timing(job_details) || job_details.weaves >= 1);

        // ALL oGCD management
        if (next_spell_name == "")
            && normal_ogcd_allowed_conditions
            && !ogcd_restricted_because_paeon
        {
            // Songs: Wanderer's Minuet 42s, Mage's Ballad 42s, Army's Paeon 33s
            // (song ticks happen every 3s, no proc in the last 3s)
            // TODO use Wanderer's Minuet as late as possible in the oGCD window so Enhanced Army's Paeon covers five GCDs
            // TODO add more safety to avoid being song-less
            if next_spell_name == "" {
                if spell::is_spell_available_now(job_details, "the Wanderer's Minuet") {
                    next_spell_name = "the Wanderer's Minuet";
                } else if !spell::is_status_active_next_gcd(job_details, "Wanderer song") {
                    // WARNING -Take care of Pitch perfect dump
                    if spell::is_spell_available_now(job_details, "Mage's Ballad") {
                        next_spell_name = "Mage's Ballad";
                    } else if !spell::is_status_active_next_gcd(job_details, "Mage song")
                        && spell::is_spell_available_now(job_details, "Army's Paeon")
                    {
                        next_spell_name = "Army's Paeon";
                    }
                }
            }

            // Force high priority emp arrow (WARNING EXPERIMENTAL)
            if next_spell_name == ""
                    && spell::spell_will_be_available_soon(job_details, "Empyreal Arrow", 500)
            {
                next_spell_name = "Empyreal Arrow";
            }

            // Burst oGCD
            // Burst ready function avoid too much drift between the oGCD buffs
            if next_spell_name == ""
                && !job_details.opener
                && job_details.burst_allowed
                && burst_ready(job_details)
            {
                next_spell_name = "Battle Voice";
            } else if spell::is_status_active_now(job_details, "Battle Voice Buff")
                && spell::is_spell_available_now(job_details, "Radiant Finale")
            {
                next_spell_name = "Radiant Finale";
            } else if spell::is_status_active_now(job_details, "Radiant Finale Buff")
                && spell::is_spell_available_now(job_details, "Raging Strikes")
            {
                next_spell_name = "Raging Strikes";
            }
            // Activate barrage if all other are active
            else if spell::is_status_active_now(job_details, "Raging Strikes Buff")
                && spell::is_spell_available_now(job_details, "Barrage")
            {
                next_spell_name = "Barrage";
            }

            // Wanderer's song procs
            // Try to prio not drifting empy arrow.
            if next_spell_name == "" && spell::is_status_active_now(job_details, "Wanderer song") {
                // Pitch perfect usage
                if next_spell_name == "" && job_details.stacks_a >= 1 {
                    // Pitch perfect dump before end of Battle voice
                    if spell::is_status_active_now(job_details, "Battle Voice Buff")
                        && !spell::is_status_active_in(job_details, "Battle Voice Buff", 2500)
                        && !spell::is_spell_available_now(job_details, "Empyreal Arrow")
                    {
                        next_spell_name = "Pitch Perfect";
                    }
                    // Pitch perfect dump before end of song
                    else if !spell::is_status_active_in(job_details, "Wanderer song", 4000) {
                        next_spell_name = "Pitch Perfect";
                    }
                    // Pitch perfect dump at full stack
                    else if job_details.stacks_a >= 3
                        && !spell::is_spell_available_now(job_details, "Empyreal Arrow")
                    {
                        next_spell_name = "Pitch Perfect";
                    }
                }
            }

            // out of burst oGCD
            if next_spell_name == "" {
                // Fix Overcap on Bloodletter
                // FIXME
                //proc

                // Empyreal Arrow on CD
                if spell::is_spell_available_now(job_details, "Empyreal Arrow") {
                    next_spell_name = "Empyreal Arrow";
                }
                // Sidewinder on CD, or wait for Barrage (burst)
                // correction : wait for raging strikes and ajust to the duration of raging strikes - 1 gcd
                // CD = 60s - 1 gcd 2.5s - raging strikes duration 20s = 38.5s
                else if (spell::is_spell_available_now(job_details, "Sidewinder")
                    && !spell::spell_will_be_available_soon(job_details, "Raging Strikes", 38500))
                    || (spell::is_spell_available_now(job_details, "Sidewinder")
                        && job_details.dump_resources)
                {
                    next_spell_name = "Sidewinder";
                }
                // Avoid bloodletter overcap
                // Stupid bloodletter spam, exept during Army song
                // "do not use if the song timer is 35 or less" can be interporeted 35 s left on peaon
                // means around 30s left on expected raging strikes CD
                else if spell::is_spell_available_now(job_details, "Bloodletter")
                    && spell::is_spell_available_now(job_details, "Heartbreak Shot")
                    && !(spell::spell_will_be_available_soon(job_details, "Raging Strikes", 30000)
                        && !job_details.dump_resources)
                {
                    if aoe {
                        next_spell_name = "Rain of Death";
                    } else {
                        next_spell_name = "Heartbreak Shot";
                    }
                }
            }
        }

        // No oGCD selected, prepare next GCD

        // No dot - do dot - emergency dot
        // No dot - do dot - emergency dot
        // Note : bit of a double with another iron jaw at 3 GCD

        // check standart GCD
        if next_spell_name == "" {
            // Emergency refresh
            if iron_jaw_castable
                && (!spell::is_status_still_active_in_two_gcd(job_details, "Storm DoT")
                    || !spell::is_status_still_active_in_two_gcd(job_details, "Caustic DoT"))
            {
                next_spell_name = "Iron Jaws";
            }
            // refresh end of burst if Dot were applied before it
            else if should_refresh_dot_end_of_burst(job_details, aoe, no_dot_rotation) {
                next_spell_name = "Iron Jaws";
            }
            // High priority on spending procs
            else if spell::is_status_active_next_gcd(job_details, "Hawk's Eye")
                || spell::is_status_active_next_gcd(job_details, "Barrage Buff")
            {
                if aoe {
                    next_spell_name = "Shadowbite";
                } else {
                    next_spell_name = "Refulgent Arrow";
                }
            }
            // Apex follow up
            else if spell::is_status_active_next_gcd(job_details, "Blast Arrow Ready") {
                next_spell_name = "Blast Arrow";
            }
            // Full soul gauge -- and no burst coming
            // use before timer on mage ballad hits 18 = paon is available in 18
            // 55s before Raging strikes to have in Burst
            else if job_details.secondary_bar >= 80 && no_burst_soon(job_details, 55000) {
                next_spell_name = "Apex Arrow";
            }
            // Follow up of burst buff
            // Radiant encore is delayed after barrage in the burst to have more friendly buff active
            else if spell::is_status_active_next_gcd(job_details, "Radiant Encore Ready")
                && !spell::spell_will_be_available_soon(job_details, "Barrage", 15000)
            {
                next_spell_name = "Radiant Encore"
            }
            // Follow up of burst buff Barrage
            else if spell::is_status_active_next_gcd(job_details, "Resonant Arrow Ready") {
                next_spell_name = "Resonant Arrow"
            }
            // Normal Doting
            else if dot_allowed && !spell::is_status_active_next_gcd(job_details, "Storm DoT") {
                next_spell_name = "Stormbite";
            } else if dot_allowed && !spell::is_status_active_next_gcd(job_details, "Caustic DoT") {
                next_spell_name = "Caustic Bite";
            }
            // Normal refresh
            else if iron_jaw_castable
                && (!spell::is_status_active_in(job_details, "Storm DoT", 7500)
                    || !spell::is_status_active_in(job_details, "Caustic DoT", 7500))
            {
                next_spell_name = "Iron Jaws";
            }
            // Filler
            else if aoe {
                next_spell_name = "Ladonsbite"
            } else {
                next_spell_name = "Burst Shot";
            }
        }

        if next_spell_name != "" {
            memory::communicate_next_spellcast(job_details, spell_cast_type, next_spell_name, true);
        }
    } else {
        error!("JOB not found in spell book");
    }
}

pub fn apply_spell_additional_effects(spell_name: &str, job_details: &mut JobDetails) {
    match spell_name {
        "Refulgent Arrow" => {
            if spell::is_status_active_now(job_details, "Barrage Buff") {
                spell::deactivate_status(job_details, "Barrage Buff")
            } else {
                spell::deactivate_status(job_details, "Hawk's Eye")
            }
        }
        "Shadowbite" => {
            if spell::is_status_active_now(job_details, "Barrage Buff") {
                spell::deactivate_status(job_details, "Barrage Buff")
            } else {
                spell::deactivate_status(job_details, "Hawk's Eye")
            }
        }
        // Battle voice is the burst trigger
        "Battle Voice" => {
            job_details.burst_allowed = true;
            job_details.dump_resources = false;
        }
        // Break opener after pot usage
        "Burst Shot" => {
            job_details.opener = false;
        }
        // If using AOE, skip opener
        "Ladonsbite" => {
            job_details.opener = false;
        }
        _ => {}
    }
}

pub fn update_job_gauge(job_details: &mut JobDetails, data: &str) {
    debug!("Job gauge update: {}", data);
    let chunks: Vec<String> = data
        .as_bytes() // Convert the string to a byte slice
        .chunks(2) // Split into chunks of 2 bytes
        .map(|chunk| String::from_utf8_lossy(chunk).to_string()) // Convert each chunk back to a String
        .collect();

    let chunk7: i32 = i32::from_str_radix(&chunks[6], 16).expect("Failed to convert chunk7");
    let chunk8: i32 = i32::from_str_radix(&chunks[7], 16).expect("Failed to convert chunk8");

    job_details.secondary_bar = chunk8; // Apex bar
    job_details.stacks_a = chunk7; // Pitch Perfect

    // Gauge layout (one byte per chunk, in order):
    // 0x00-0x01 songMilliseconds (u16)
    // 0x02 songProcs
    // 0x03 soulGauge
    // 0x04 song type (None = 0, Ballad = 5, Paeon = 10, Minuet = 15)
    // chunks[5] = Pitch procs
}

fn burst_ready(job_details: &mut JobDetails) -> bool {
    //     Battle voice
    // 2.5 radiant Finale
    // 3 Raging Strike
    // 5 barrage
    let battle_voice_ready = spell::is_spell_available_now(job_details, "Battle Voice");
    let radiant_finale_in_line =
        spell::spell_will_be_available_soon(job_details, "Radiant Finale", 2750); // Should be ready next GCD window
    let raging_strike_in_line =
        spell::spell_will_be_available_soon(job_details, "Radiant Finale", 3500); // Worst case the next next GCD window
    let barrage_in_line = spell::spell_will_be_available_soon(job_details, "Radiant Finale", 6000); // in Two GCD window, worst case 3

    return battle_voice_ready
        && radiant_finale_in_line
        && raging_strike_in_line
        && barrage_in_line;
}

fn are_dot_allowed(job_details: &mut JobDetails, aoe: bool, no_dot_rotation: bool) -> bool {
    return !aoe
        && !no_dot_rotation
        && !job_details.dump_resources
        && !spell::is_status_active_now(job_details, "Do Not Dot");
}

fn is_iron_jaws_castable(job_details: &mut JobDetails, aoe: bool, no_dot_rotation: bool) -> bool {
    let storm_dot_active = spell::is_status_active_next_gcd(job_details, "Storm DoT");
    let caustic_dot_active = spell::is_status_active_next_gcd(job_details, "Caustic DoT");
    let dot_allowed: bool = are_dot_allowed(job_details, aoe, no_dot_rotation);
    return !aoe && dot_allowed && storm_dot_active && caustic_dot_active;
}

// refresh_in_burst
// If end of burst and no recent refresh, refresh Dots
// recent usage = last 7 GCD ~
// So if dot finish in more than 45s - 7 gcd = about 27.5s

fn should_refresh_dot_end_of_burst(
    job_details: &mut JobDetails,
    aoe: bool,
    no_dot_rotation: bool,
) -> bool {
    let end_of_burst = spell::is_status_active_now(job_details, "Battle Voice Buff")
        && !spell::is_status_still_active_in_two_gcd(job_details, "Battle Voice Buff");
    let storm_applied_before_burst = !spell::is_status_active_in(job_details, "Storm DoT", 27500);
    let caustic_applied_before_burst: bool =
        !spell::is_status_active_in(job_details, "Caustic DoT", 27500);
    let iron_jaws_castable = is_iron_jaws_castable(job_details, aoe, no_dot_rotation);

    return end_of_burst
        && storm_applied_before_burst
        && caustic_applied_before_burst
        && iron_jaws_castable;
}

// used to postponed cast if burst is coming soon
// Used for resources better saved if a burst window is coming
// Burst is ignored if "dump ressources" is active
// raging strike because we wait to have at least all our own buff and it is the last one cast
fn no_burst_soon(job_details: &mut JobDetails, max_waiting_duration: i32) -> bool {
    let raging_strike_soon =
        spell::spell_will_be_available_soon(job_details, "Raging Strikes", max_waiting_duration);
    return job_details.dump_resources || !raging_strike_soon;
}
