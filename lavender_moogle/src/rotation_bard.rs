// rotation_bard.rs
// Rotation logic for bard job

use log::{debug, error};

use crate::memory;
use crate::spell;
use crate::types::{CharacterDetails, JobDetails};
use chrono::Utc;

pub fn update_next_spellcast(character_details: &mut CharacterDetails, aoe: bool, alternate: bool) {
    if let Some(job_details) = character_details.get_mut("BRD") {
        const MAX_WEAVE_NUMBER: i32 = 2;
        // update status
        // check oGCD
        let mut next_spell_name = "";
        // let burst_allowed = spell::burst_allowed(job_details);
        // let delayed_burst = !burst_allowed;

        // update stack min / max

        // if job_details.stacks_a > 4 {
        //     job_details.stacks_a = 4;
        // } else if job_details.stacks_a < 0 {
        //     job_details.stacks_a = 0;
        // }

        // Hack for starting with HB shot, currently no idea of the charges
        // if job_details.opener
        //     && job_details.stacks_b == 0
        //     && (job_details.weaves < MAX_WEAVE_NUMBER)
        // {
        //     next_spell_name = "Heartbreak Shot";
        // }

        // if job_details.opener && should_end_opener(job_details) {
        //     debug!("[OPENER_TRACK] Ending opener based on Burst Shot last_cast timestamp");
        //     job_details.opener = false;
        // }

        // Hack to detect the very first cast of the opener and do the HB shot
        // We should do a GCD (no oGCD) and wanderer not started
        // Ca ne marche pas, comment ne le ffaire qu'une fois ? il faut detecter qu'il n'y a jamais eu de GCD
        if job_details.opener && !aoe && !alternate {
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
            } else if spell::is_spell_available_now(job_details, "Battle Voice") {
                next_spell_name = "Battle Voice";
            } else {
                next_spell_name = "Burst Shot";
            }
        }

        // ALL oGCD management
        if (next_spell_name == "")
            && (job_details.weaves < MAX_WEAVE_NUMBER)
            && spell::ogcd_end_before_next_gcd(job_details)
        {
            // BURST oGCD
            // FIXME

            // OPENER
            // FIXME - find a hack to start with the oGCD HB
            // if next_spell_name == "" && job_details.opener {
            //     if spell::is_spell_available_now(job_details, "the Wanderer's Minuet") {
            //         next_spell_name = "the Wanderer's Minuet";
            //     } else if spell::is_spell_available_now(job_details, "Empyreal Arrow") {
            //         next_spell_name = "Empyreal Arrow";
            //     } else if spell::is_spell_available_now(job_details, "Battle Voice") {
            //         next_spell_name = "Battle Voice";
            //     }
            //     // Auto pot broken - rather skip than doing weird
            //     // else {
            //     //     next_spell_name = "Potion";
            //     // }
            // } else {
            // Songs
            if next_spell_name == ""
                && spell::is_spell_available_now(job_details, "the Wanderer's Minuet")
            {
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
            // }

            // // BUFF ACTIVATION
            // // Activate Battle Voice 1 GCD after Minuet
            // if spell::spell_will_be_available_soon(job_details, "Wanderer song", 118250) {
            //     if spell::is_spell_available_now(job_details, "Battle Voice") {
            //         next_spell_name = "Battle Voice";
            //     }
            //     // Activate Radiant Finale and Raging strike 2 GCD after Minuet
            //     else if spell::spell_will_be_available_soon(job_details, "Wanderer song", 115850)
            //     {
            //         if spell::is_spell_available_now(job_details, "Radiant Finale") {
            //             next_spell_name = "Radiant Finale";
            //         }
            //         // Activate Raging Strike 2 GCD after Radiant Finale
            //         else if spell::is_spell_available_now(job_details, "Raging Strikes") {
            //             next_spell_name = "Raging Strikes";
            //         }
            //         // Activate barrage if all other are active
            //         else if spell::is_spell_available_now(job_details, "Barrage") {
            //             next_spell_name = "Barrage";
            //         }
            //     }
            // }
            // Opener special case
            // if next_spell_name == "" && job_details.opener {
            //     if spell::is_status_active_now(job_details, "Wanderer song") {
            //         if spell::is_spell_available_now(job_details, "Empyreal Arrow") {
            //             next_spell_name = "Empyreal Arrow";
            //         } else if spell::is_spell_available_now(job_details, "Battle Voice") {
            //             next_spell_name = "Battle Voice";
            //         } else {
            //             next_spell_name = "Potion";
            //         }
            //     }
            // }
            // BURST BUFF ACTIVATION
            // Activate Battle Voice 1 GCD after Minuet

            // Force high priority emp arrow (WARNING EXPERIMENTAL)
            if next_spell_name == ""
                    // && spell::is_spell_available_now(job_details, "Empyreal Arrow")
                    && spell::spell_will_be_available_soon(job_details, "Empyreal Arrow", 300)
            {
                next_spell_name = "Empyreal Arrow";
            }

            if next_spell_name == ""
                && !job_details.opener
                && job_details.burst_allowed
                && spell::is_spell_available_now(job_details, "Battle Voice")
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
                else if spell::is_spell_available_now(job_details, "Sidewinder")
                    && !spell::spell_will_be_available_soon(job_details, "Barrage", 52500)
                {
                    next_spell_name = "Sidewinder";
                }
                // Avoid bloodletter overcap
                // Stupid bloodletter spam, exept during Army song
                else if spell::is_spell_available_now(job_details, "Bloodletter")
                    && spell::is_spell_available_now(job_details, "Heartbreak Shot")
                    && !(spell::spell_will_be_available_soon(job_details, "Barrage", 35000)
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
        if next_spell_name == "" && !alternate && !aoe {
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
        if next_spell_name == "" {
            // Refresh buffed Iron Jaws before end of burst
            // FIXME : Should be done only if we didn't already refresh in burst
            // Hack : "recent refresh buff"
            if spell::is_status_active_next_gcd(job_details, "Battle Voice Buff")
                && !spell::is_status_still_active_in_two_gcd(job_details, "Battle Voice Buff")
                && !spell::is_status_active_next_gcd(job_details, "Recent Refresh")
                && !alternate
                && !aoe
            {
                next_spell_name = "Iron Jaws";
            }
            // Delay if burst is coming
            else if (spell::is_status_active_next_gcd(job_details, "Hawk's Eye")
                && !spell::spell_will_be_available_soon(job_details, "Raging Strikes", 7500))
                || spell::is_status_active_next_gcd(job_details, "Barrage Buff")
            {
                if aoe {
                    next_spell_name = "Shadowbite";
                } else {
                    next_spell_name = "Refulgent Arrow";
                }
            }
            // Blast arraow - follow up apex
            else if spell::is_status_active_next_gcd(job_details, "Blast Arrow Ready") {
                next_spell_name = "Blast Arrow";
            }
            // Full soul gauge -- and no burst coming
            else if job_details.secondary_bar >= 80
                && !spell::spell_will_be_available_soon(job_details, "Raging Strikes", 15000)
            {
                next_spell_name = "Apex Arrow";
            } else if spell::is_status_active_next_gcd(job_details, "Radiant Encore Ready")
            // && !spell::spell_will_be_available_soon(job_details, "Barrage", 5000)
            && !spell::spell_will_be_available_soon(job_details, "Barrage", 15000)
            // && spell::is_status_active_next_gcd(job_details, "Barrage Buff")
            // && spell::is_status_active_next_gcd(job_details, "Radiant Finale Buff")
            // && spell::is_status_active_next_gcd(job_details, "Raging Strikes Buff")
            // && spell::is_status_active_next_gcd(job_details, "Battle Voice Buff")
            {
                // Big hit rdiant encore, following radiant finale. Wait to be fully under burst
                next_spell_name = "Radiant Encore"
            } else if spell::is_status_active_next_gcd(job_details, "Resonant Arrow Ready")
            // && spell::is_status_active_next_gcd(job_details, "Barrage Buff")
            // && spell::is_status_active_next_gcd(job_details, "Radiant Finale Buff")
            // && spell::is_status_active_next_gcd(job_details, "Raging Strikes Buff")
            // && spell::is_status_active_next_gcd(job_details, "Battle Voice Buff")
            {
                // Big hit rdiant encore, following radiant finale. Wait to be fully under burst
                next_spell_name = "Resonant Arrow"
            }
            // Prevent Dot Drop
            else if (!spell::is_status_active_in(job_details, "Storm DoT", 7500)
                || !spell::is_status_active_in(job_details, "Caustic DoT", 7500))
                && !alternate
                && !aoe
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
            memory::communicate_next_spellcast(job_details, next_spell_name, aoe, alternate);
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

    // let chunk1: i32 = i32::from_str_radix(&chunks[0], 16).expect("Failed to convert chunk1");
    // let chunk2: i32 = i32::from_str_radix(&chunks[1], 16).expect("Failed to convert chunk2");
    // let chunk3: i32 = i32::from_str_radix(&chunks[2], 16).expect("Failed to convert chunk3");
    // let chunk4: i32 = i32::from_str_radix(&chunks[3], 16).expect("Failed to convert chunk4");
    // let chunk5: i32 = i32::from_str_radix(&chunks[4], 16).expect("Failed to convert chunk5");
    // let chunk6: i32 = i32::from_str_radix(&chunks[5], 16).expect("Failed to convert chunk6");
    let chunk7: i32 = i32::from_str_radix(&chunks[6], 16).expect("Failed to convert chunk7");
    let chunk8: i32 = i32::from_str_radix(&chunks[7], 16).expect("Failed to convert chunk8");
    // let chunk9: i32 = i32::from_str_radix(&chunks[8], 16).expect("Failed to convert chunk9");
    // let chunk10: i32 = i32::from_str_radix(&chunks[9], 16).expect("Failed to convert chunk10");
    // let chunk11: i32 = i32::from_str_radix(&chunks[10], 16).expect("Failed to convert chunk11");
    // let chunk12: i32 = i32::from_str_radix(&chunks[11], 16).expect("Failed to convert chunk12");
    // let chunk13: i32 = i32::from_str_radix(&chunks[12], 16).expect("Failed to convert chunk13");
    // let chunk14: i32 = i32::from_str_radix(&chunks[13], 16).expect("Failed to convert chunk14");
    // let chunk15: i32 = i32::from_str_radix(&chunks[14], 16).expect("Failed to convert chunk15");
    // let chunk16: i32 = i32::from_str_radix(&chunks[15], 16).expect("Failed to convert chunk16");

    job_details.secondary_bar = chunk8; // Apex bar
    job_details.stacks_a = chunk7; //Pitch Perfect

    // // Debug output for each chunk
    // debug!("chunk1 = {} songMilliseconds", chunk1);
    // debug!("chunk2 = {} songProcs", chunk2);
    // debug!("chunk3 = {} soulGauge", chunk3);
    // debug!("chunk4 = {} song_type", chunk4);
    // debug!("chunk5 = {}", chunk5);
    // debug!("chunk6 = {}", chunk6); // Pitch procs
    // debug!("chunk7 = {}", chunk7); // Soul voice
    // debug!("chunk8 = {}", chunk8);
    // debug!("chunk9 = {}", chunk9);
    // debug!("chunk10 = {}", chunk10);
    // debug!("chunk11 = {}", chunk11);
    // debug!("chunk12 = {}", chunk12);
    // debug!("chunk13 = {}", chunk13);
    // debug!("chunk14 = {}", chunk14);
    // debug!("chunk15 = {}", chunk15);
    // debug!("chunk16 = {}", chunk16);

    // public struct BardJobMemory {
    //     private enum Song : byte {
    //       None = 0,
    //       Ballad = 5, // Mage's Ballad.
    //       Paeon = 10, // Army's Paeon.
    //       Minuet = 15, // The Wanderer's Minuet.
    //     }

    //     [FieldOffset(0x00)]
    //     public ushort songMilliseconds;

    //     [FieldOffset(0x02)]
    //     public byte songProcs;

    //     [FieldOffset(0x03)]
    //     public byte soulGauge;

    //     [NonSerialized]
    //     [FieldOffset(0x04)]
    //     private Song song_type;

    //     public String songName {
    //       get {
    //         return !Enum.IsDefined(typeof(Song), song_type) ? "None" : song_type.ToString();
    //       }
    //     }

    //   };
}

// Names for completion

// Burst Shot
// Hawk's Eye
// Refulgent Arrow
// Apex Arrow
// Blast Arrow Ready
// Blast Arrow
// Ladonsbite
// Shadowbite

// Resonant Arrow Ready
// Resonant Arrow
// Radiant Encore Ready
// Radiant Encore

// Stormbite
// Storm DoT
// Caustic Bite
// Caustic DoT
// Iron Jaws

// the Wanderer's Minuet
// Wanderer song
// Pitch Perfect

// Mage's Ballad
// Mage song
// Bloodletter
// Heartbreak Shot
// Rain of Death

// Army's Paeon
// Army song
// Enhanced Army’s Paeon

// Barrage
// Barrage Buff
// Battle Voice
// Battle Voice Buff
// Raging Strikes
// Raging Strikes Buff
// Radiant Finale
// Radiant Finale Buff

// Sidewinder
// Empyreal Arrow
