// rotation_summoner.rs
// Rotation logic for summoner job

use log::{debug, error};

use crate::memory;
use crate::spell;
use crate::types::{CharacterDetails, JobDetails, SpellcastType};

pub fn update_next_spellcast(
    character_details: &mut CharacterDetails,
    spell_cast_type: SpellcastType,
    aoe_old: bool,
    alternate_old: bool,
) {
    if let Some(job_details) = character_details.get_mut("SMN") {
        const MAX_WEAVE_NUMBER: i32 = 2;
        // update status

        // VARIATIONS CASTERS
        // 1 : Constrained mono casts
        // 2 : Instants only - free movement
        // 3 : Constrained AOE casts. In the case of Scholar, no restriction. This is an exception
        // 4 : Classic - mono - user can interrupt cast by moving.
        // There is a question to make the charges dump related to this fourth choice.
        // It would be for chill play and do the dump only then, as to not make a mess in managed mode

        let mut aoe = false;
        let mut restricted_movement_allowed = false; // FIXME this variable name is horrible
        let mut instant_cast_only = false;

        match spell_cast_type {
            SpellcastType::First => restricted_movement_allowed = true,
            SpellcastType::Second => instant_cast_only = true,
            SpellcastType::Third => aoe = true,
            SpellcastType::Fourth => (),
        }

        // Todo both should be managed by job gauge
        // update stack min / max
        // Stack a = aetherflow
        if job_details.stacks_a > 2 {
            job_details.stacks_a = 2;
        } else if job_details.stacks_a < 0 {
            job_details.stacks_a = 0;
        }

        // Stack b = attunments
        if job_details.stacks_b > 4 {
            job_details.stacks_b = 4;
        } else if job_details.stacks_b < 0 {
            job_details.stacks_b = 0;
        }

        let mut next_spell_name = "";
        let burst_allowed = spell::burst_allowed(job_details);

        // check oGCD
        if (job_details.weaves < MAX_WEAVE_NUMBER) && spell::ogcd_end_before_next_gcd(job_details) {
            if job_details.opener {
                // In Baha since more than 1 x GCD ? buff dispo ? -> buff
                if spell::is_status_active_since_x_gcd(job_details, "Big Summon Cooldown", 1)
                    && spell::is_spell_available_now(job_details, "Searing Light")
                    && burst_allowed
                {
                    next_spell_name = "Searing Light";
                }
                // In Baha since more than 3 x GCD ? drain dispo -> drain, stop opener
                else if spell::is_status_active_since_x_gcd(job_details, "Big Summon Cooldown", 3)
                    && is_drain_available(job_details)
                {
                    if aoe {
                        next_spell_name = "Energy Siphon";
                    } else {
                        next_spell_name = "Energy Drain";
                    }

                    // Stop opener special trigger after Drain and enough GCD.
                } else if spell::is_status_active_since_x_gcd(job_details, "Big Summon Cooldown", 4)
                    && !is_drain_available(job_details)
                {
                    job_details.opener = false;
                } else
                // In Baha, pot dispo ?  pot
                if spell::is_status_active_now(job_details, "Big Summon Cooldown")
                    && spell::is_spell_available_now(job_details, "Potion")
                {
                    next_spell_name = "Potion";
                }
                // end of opener treatment
            } else
            // --- A Priority: no drifting raid buffs
            if spell::is_spell_available_now(job_details, "Searing Light") && burst_allowed {
                next_spell_name = "Searing Light";

            // --- Aerth charges
            } else if spell::is_status_active_now(job_details, "Searing Light Buff")
                && job_details.stacks_a > 0
            {
                if aoe {
                    next_spell_name = "Painflare";
                } else {
                    next_spell_name = "Necrotize";
                }
            } else if is_drain_available(job_details) && job_details.stacks_a <= 0 {
                if aoe {
                    next_spell_name = "Energy Siphon";
                } else {
                    next_spell_name = "Energy Drain";
                }
            // -- Summons special actions
            } else if spell::is_status_active_now(job_details, "Mountain Buster Ready") {
                next_spell_name = "Mountain Buster";
            } else if spell::is_status_active_now(job_details, "Ekindle Ready") {
                next_spell_name = "Enkindle Bahamut";
            } else if should_flare(job_details) {
                next_spell_name = "Deathflare";
            } else if spell::is_status_active_now(job_details, "Ruby's Glimmer") {
                next_spell_name = "Searing Flash";
            } else if should_swiftcast(job_details, instant_cast_only) {
                next_spell_name = "Swiftcast";
            } else if should_lux(job_details) {
                next_spell_name = "Lux Solaris";
            }
        }

        // No oGCD selected, prepare next GCD

        // no check on EXPIRING GCD because not an easy occurance for this job

        // check normal GCD
        if next_spell_name == "" {
            // Swiftcast has been activated for garuda
            if should_swiftcast_splitstream(job_details) {
                next_spell_name = "Slipstream";

            // big summons
            } else if !spell::is_status_active_next_gcd(job_details, "Big Summon Cooldown") {
                next_spell_name = "Summon Bahamut";
            } else if in_big_primal_phase_next_gcd(job_details) {
                if aoe {
                    next_spell_name = "Tri-disaster";
                } else {
                    next_spell_name = "Ruin III";
                }
            // lesser summons
            } else if spell::is_status_active_next_gcd(job_details, "Titan Ready")
                && !is_lesser_summon_in_process(job_details)
            {
                next_spell_name = "Summon Titan II";
            } else if spell::is_status_active_next_gcd(job_details, "Garuda Ready")
                && !is_lesser_summon_in_process(job_details)
            {
                next_spell_name = "Summon Garuda II";
            } else if spell::is_status_active_next_gcd(job_details, "Ifrit Ready")
                && !is_lesser_summon_in_process(job_details)
            {
                next_spell_name = "Summon Ifrit II";

            // Swiftcast has been activated for garuda
            } else if should_swiftcast_splitstream(job_details) {
                next_spell_name = "Slipstream";

            // If Crimson Stike is ready, Crimson Cyclone has been manually casted
            // And we should be in melee
            // WARNING :// Crimson Cyclone Ready and Crimson Cyclone ignored from auto mod
            } else if spell::is_status_active_next_gcd(job_details, "Crimson Strike Ready") {
                next_spell_name = "Crimson Strike";

            // Attenument usage - same for all lesser summons
            // TODO - should manage fire instant cast only
            // BUT in this case there is 0 Instant cast available. instant_cast_only is not tested because we don't have anything better than trying to cast this anyway
            } else if job_details.stacks_b > 0 {
                if aoe {
                    next_spell_name = "Precious Brilliance";
                } else {
                    next_spell_name = "Gemshine";
                }

            // hard-casted slipstream
            } else if spell::is_status_active_next_gcd(job_details, "Slipstream Ready")
                && !instant_cast_only
            {
                next_spell_name = "Slipstream";
                // FIXME FIXME BROKEN
            } else if spell::is_status_active_next_gcd(job_details, "Further Ruin") {
                next_spell_name = "Ruin IV";
            } else {
                // WARNING in this case there is 0 Instant cast available. instant_cast_only is not tested because we don't have anything better than trying to cast this anyway
                if aoe {
                    next_spell_name = "Tri-disaster";
                } else {
                    next_spell_name = "Ruin III";
                }
            }
        }

        if next_spell_name != "" {
            memory::communicate_next_spellcast(job_details, spell_cast_type, next_spell_name);
        }
    } else {
        error!("JOB not found in spell book");
    }
}

fn in_big_primal_phase_next_gcd(job_details: &mut JobDetails) -> bool {
    return spell::is_status_active_next_gcd(job_details, "Dreadwyrm Trance")
        || spell::is_status_active_next_gcd(job_details, "Firebird Trance")
        || spell::is_status_active_next_gcd(job_details, "Lightwyrm Trance");
}

fn in_dreadwyrm_phase(job_details: &mut JobDetails) -> bool {
    return spell::is_status_active_now(job_details, "Dreadwyrm Trance");
}

fn in_pheonix_phase(job_details: &mut JobDetails) -> bool {
    return spell::is_status_active_now(job_details, "Firebird Trance");
}

fn in_solar_phase(job_details: &mut JobDetails) -> bool {
    return spell::is_status_active_now(job_details, "Lightwyrm Trance");
}

fn should_flare(job_details: &mut JobDetails) -> bool {
    if spell::is_status_active_now(job_details, "Flare Ready") {
        if in_dreadwyrm_phase(job_details) || in_solar_phase(job_details) {
            return true;
        } else if in_pheonix_phase(job_details)
            && !spell::is_status_still_active_in_two_gcd(job_details, "Firebird Trance")
        {
            return true;
        }
    }
    false
}

fn is_ifrit_in_process(job_details: &mut JobDetails) -> bool {
    if spell::is_status_active_now(job_details, "Ifrit") {
        return spell::is_status_active_now(job_details, "Crimson Cyclone Ready")
            || spell::is_status_active_now(job_details, "Crimson Strike Ready")
            || job_details.stacks_b > 0;
    }
    false
}

fn is_titan_in_process(job_details: &mut JobDetails) -> bool {
    if spell::is_status_active_now(job_details, "Titan") {
        return spell::is_status_active_now(job_details, "Mountain Buster Ready")
            || job_details.stacks_b > 0;
    }
    false
}

fn is_garuda_in_process(job_details: &mut JobDetails) -> bool {
    if spell::is_status_active_now(job_details, "Garuda") {
        return spell::is_status_active_now(job_details, "Slipstream Ready")
            || job_details.stacks_b > 0;
    }
    false
}

fn is_lesser_summon_in_process(job_details: &mut JobDetails) -> bool {
    return is_titan_in_process(job_details)
        || is_garuda_in_process(job_details)
        || is_ifrit_in_process(job_details);
}

fn should_lux(job_details: &mut JobDetails) -> bool {
    let lux_available = spell::is_status_active_now(job_details, "Lux Solaris Ready");
    let lux_will_go = !spell::is_status_still_active_in_two_gcd(job_details, "Lux Solaris Ready");
    return lux_available && lux_will_go;
}

fn should_swiftcast(job_details: &mut JobDetails, instant_cast_only: bool) -> bool {
    let slipstream_ready = spell::is_status_active_next_gcd(job_details, "Slipstream Ready");
    let garuda_active = spell::is_status_active_next_gcd(job_details, "Garuda");
    let swiftcast_available = spell::is_spell_available_now(job_details, "Swiftcast");
    return slipstream_ready && garuda_active && swiftcast_available && instant_cast_only;
    // return false;
}

fn should_swiftcast_splitstream(job_details: &mut JobDetails) -> bool {
    let slipstream_ready = spell::is_status_active_next_gcd(job_details, "Slipstream Ready");
    let garuda_active = spell::is_status_active_next_gcd(job_details, "Garuda");
    let swiftcast_active = spell::is_status_active_next_gcd(job_details, "Swiftcast Buff");

    return slipstream_ready && garuda_active && swiftcast_active;
    // return false
}

fn is_drain_available(job_details: &mut JobDetails) -> bool {
    return spell::is_spell_available_now(job_details, "Energy Drain")
        && spell::is_spell_available_now(job_details, "Energy Siphon");
}

pub fn apply_spell_additional_effects(spell_name: &str, job_details: &mut JobDetails) {
    match spell_name {
        // Attenument stacks
        "Summon Ifrit II" => job_details.stacks_b = 2,
        "Summon Titan II" => job_details.stacks_b = 4,
        "Summon Garuda II" => job_details.stacks_b = 4,
        "Summon Bahamut" => job_details.stacks_b = 0,
        "Summon Phoenix" => job_details.stacks_b = 0,
        "Summon Solar Bahamut" => job_details.stacks_b = 0,
        _ => {}
    }
}

pub fn update_job_gauge(_job_details: &mut JobDetails, data: &str) {
    debug!("Job gauge update: {}", data);
    let chunks: Vec<String> = data
        .as_bytes() // Convert the string to a byte slice
        .chunks(2) // Split into chunks of 2 bytes
        .map(|chunk| String::from_utf8_lossy(chunk).to_string()) // Convert each chunk back to a String
        .collect();

    let chunk1: i32 = i32::from_str_radix(&chunks[0], 16).expect("Failed to convert chunk1");
    let chunk2: i32 = i32::from_str_radix(&chunks[1], 16).expect("Failed to convert chunk2");
    let chunk3: i32 = i32::from_str_radix(&chunks[2], 16).expect("Failed to convert chunk3");
    let chunk4: i32 = i32::from_str_radix(&chunks[3], 16).expect("Failed to convert chunk4");
    let chunk5: i32 = i32::from_str_radix(&chunks[4], 16).expect("Failed to convert chunk5");
    let chunk6: i32 = i32::from_str_radix(&chunks[5], 16).expect("Failed to convert chunk6");
    let chunk7: i32 = i32::from_str_radix(&chunks[6], 16).expect("Failed to convert chunk7");
    let chunk8: i32 = i32::from_str_radix(&chunks[7], 16).expect("Failed to convert chunk8");
    let chunk9: i32 = i32::from_str_radix(&chunks[8], 16).expect("Failed to convert chunk9");
    let chunk10: i32 = i32::from_str_radix(&chunks[9], 16).expect("Failed to convert chunk10");
    let chunk11: i32 = i32::from_str_radix(&chunks[10], 16).expect("Failed to convert chunk11");
    let chunk12: i32 = i32::from_str_radix(&chunks[11], 16).expect("Failed to convert chunk12");
    let chunk13: i32 = i32::from_str_radix(&chunks[12], 16).expect("Failed to convert chunk13");
    let chunk14: i32 = i32::from_str_radix(&chunks[13], 16).expect("Failed to convert chunk14");
    let chunk15: i32 = i32::from_str_radix(&chunks[14], 16).expect("Failed to convert chunk15");
    let chunk16: i32 = i32::from_str_radix(&chunks[15], 16).expect("Failed to convert chunk16");

    // attenument garuda is a byte half on chunk
    debug!("attnment ? = {:#x}", chunk8);
    debug!("aether ? = {:#x}", chunk4);

    // Debug output for each chunk
    debug!("chunk1 = {}", chunk1);
    debug!("chunk2 = {}", chunk2);
    debug!("chunk3 = {}", chunk3);
    debug!("chunk4 = {}", chunk4);
    debug!("chunk5 = {}", chunk5);
    debug!("chunk6 = {}", chunk6);
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

    // Parse the hex string into bytes

    // Parse the hex string into bytes
    let bytes: Vec<u8> = data
        .as_bytes()
        .chunks(2)
        .map(|chunk| {
            let hex_byte = std::str::from_utf8(chunk).unwrap_or("00");
            u8::from_str_radix(hex_byte, 16).unwrap_or(0)
        })
        .collect();

    // We're not modifying JobDetails directly as it's an external type
    // Instead, we'll just parse the bytes and output debug information

    if bytes.len() > 7 {
        // For Arcanist job memory
        let aetherflow_stacks = bytes[4];
        debug!("Arcanist - aetherflow_stacks: {}", aetherflow_stacks);

        // For Summoner job memory
        // Offset 0x00: trance_milliseconds (2 bytes)
        let trance_milliseconds = u16::from_le_bytes([bytes[0], bytes[1]]);

        // Offset 0x02: attunement_milliseconds (2 bytes)
        let attunement_milliseconds = u16::from_le_bytes([bytes[2], bytes[3]]);

        // Offset 0x06: attunement
        let attunement = bytes[6];

        // Offset 0x07: stance
        let stance = bytes[7];

        // Calculate derived properties
        let aetherflow_stacks_smn = stance & 0x3;

        let active_primal = match stance & 0xC {
            0x4 => Some("Ifrit"),
            0x8 => Some("Titan"),
            0xC => Some("Garuda"),
            _ => None,
        };

        let next_summoned = if (stance & 0x10) == 0 {
            "Bahamut"
        } else {
            "Phoenix"
        };

        let mut usable_arcanum = Vec::new();
        if (stance & 0x20) != 0 {
            usable_arcanum.push("Ruby"); // Fire/Ifrit
        }
        if (stance & 0x40) != 0 {
            usable_arcanum.push("Topaz"); // Earth/Titan
        }
        if (stance & 0x80) != 0 {
            usable_arcanum.push("Emerald"); // Wind/Garuda
        }

        // Debug output
        debug!("Summoner - trance_milliseconds: {}", trance_milliseconds);
        debug!(
            "Summoner - attunement_milliseconds: {}",
            attunement_milliseconds
        );
        debug!("Summoner - attunement: {}", attunement);
        debug!("Summoner - stance: {}", stance);
        debug!("Summoner - aetherflow_stacks: {}", aetherflow_stacks_smn);
        debug!("Summoner - active_primal: {:?}", active_primal);
        debug!("Summoner - next_summoned: {}", next_summoned);
        debug!("Summoner - usable_arcanum: {:?}", usable_arcanum);
    }
}
