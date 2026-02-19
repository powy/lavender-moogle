// rotation_scholar.rs
// Rotation logic for scholar job

use log::error;

use crate::memory;
use crate::spell;
use crate::types::{CharacterDetails, JobDetails, SpellcastType};

pub fn update_next_spellcast(
    character_details: &mut CharacterDetails,
    spell_cast_type: SpellcastType,
    _aoe: bool,
    _alternate: bool,
) {
    if let Some(job_details) = character_details.get_mut("SCH") {
        const MAX_WEAVE_NUMBER: i32 = 1;
        // ============= update status =============

        let mut next_spell_name = "";
        let burst_allowed = job_details.burst_allowed;
        let delayed_burst = !burst_allowed;

        let mut aoe = false;
        // let mut alternate = old_alternate;

        let mut can_movement_be_restricted = false; // FIXME this variable name is horrible
        let mut instant_cast_only = false;

        // VARIATIONS CASTERS
        // 1 : Constrained mono casts
        // 2 : Instants only - free movement
        // 3 : Constrained AOE casts. In the case of Scholar, no restriction. This is an exception
        // 4 : Classic - mono - user can interrupt cast by moving.
        // FIXME: 4th to be 2 targets (for healer = same)
        // Note that Aoe cast are interruptable for regen healers
        // There is a question to make the charges dump related to this fourth choice.
        // It would be for chill play and do the dump only then, as to not make a mess in managed mode

        match spell_cast_type {
            SpellcastType::First => can_movement_be_restricted = true,
            SpellcastType::Second => instant_cast_only = true,
            SpellcastType::Third => aoe = true,
            SpellcastType::Fourth => (),
        }

        // Fixme - also use spell_cast etc. from job_details
        // let static_cast_allowed = !can_movement_be_restricted;s

        // Manage stop dot and no Dot in AOE
        let dot_allowed = !aoe && !spell::is_status_active_now(job_details, "Do Not Dot");

        // ============= oGCD =============

        // ============= GCD =============
        if (job_details.weaves < MAX_WEAVE_NUMBER) && spell::ogcd_end_before_next_gcd(job_details) {
            // --- A Priority: no losing charges Refresh charges with Aetherflow
            if should_aetherflow(job_details) {
                next_spell_name = "Aetherflow";
                // -- B priority - no drifting raid buffs
            } else if spell::is_spell_available_now(job_details, "Chain Stratagem") && burst_allowed
            {
                next_spell_name = "Chain Stratagem";
            // Impact imminient is delayed because doing aetherflow is a priority - Their might be a reason
            } else if spell::is_status_active_now(job_details, "Impact Imminent")
                && !spell::is_spell_available_now(job_details, "Aetherflow")
            {
                next_spell_name = "Baneful Impaction";
            } else if should_dump_aether_charges(job_details) {
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
                } else if dot_allowed && !spell::is_status_active_in(job_details, "Dot", 3000) {
                    next_spell_name = "Biolysis";
                // Default GCD
                } else {
                    if instant_cast_only {
                        next_spell_name = "Ruin II"
                    } else {
                        next_spell_name = "Broil IV";
                    }
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

fn should_dump_aether_charges(_job_details: &mut JobDetails) -> bool {
    // If there is 1 / 2 /3 GCD left, and 1 / 2 /3 charges left, we should use them
    // Too dangerous
    return false;
    // if job_details.stacks_a > 0 {
    //     if (job_details.stacks_a >= 3)
    //         && spell::is_spell_available_in_x_gcd(job_details, "Aetherflow", 3)
    //     {
    //         return true;
    //     }
    //     if (job_details.stacks_a >= 2)
    //         && spell::is_spell_available_in_x_gcd(job_details, "Aetherflow", 2)
    //     {
    //         return true;
    //     }
    //     if (job_details.stacks_a >= 1)
    //         && spell::is_spell_available_in_x_gcd(job_details, "Aetherflow", 1)
    //     {
    //         return true;
    //     }
    // }
    // return false;
}

fn should_aetherflow(job_details: &mut JobDetails) -> bool {
    return job_details.stacks_a <= 0 && spell::is_spell_available_now(job_details, "Aetherflow");
}

// Opti suggested by guide, but makes the wound slide
fn should_reset_dot_end_of_burst(job_details: &mut JobDetails) -> bool {
    let dot_mostly_spent = !spell::is_status_active_in(job_details, "Dot", 8000); // 8s left on dot or less
    let in_burst_next_gcd = spell::is_status_active_next_gcd(job_details, "Burst");
    let in_burst_in_2_gcd = spell::is_status_still_active_in_two_gcd(job_details, "Burst");
    let end_of_burst_situation = in_burst_next_gcd && !in_burst_in_2_gcd;
    let end_of_phase = job_details.dump_resources;
    let dot_allowed = !spell::is_status_active_now(job_details, "Do Not Dot");

    return dot_allowed && end_of_burst_situation && dot_mostly_spent && !end_of_phase;
}

pub fn apply_spell_additional_effects(spell_name: &str, job_details: &mut JobDetails) {
    // manage number of weave before it gets cleanned up with new functionality in Excel
    match spell_name {
        "Broil IV" | "Summon Eos" | "Adloquium" | "Physick" | "Concitation" | "Accession"
        | "Manifestation" => {
            job_details.weaves += 1;
        }

        _ => (),
    }
}

pub fn update_job_gauge(job_details: &mut JobDetails, data: &str) {
    let chunks: Vec<String> = data
        .as_bytes() // Convert the string to a byte slice
        .chunks(2) // Split into chunks of 2 bytes
        .map(|chunk| String::from_utf8_lossy(chunk).to_string()) // Convert each chunk back to a String
        .collect();

    let aether_charges: i32 =
        i32::from_str_radix(&chunks[2], 16).expect("Failed to convert chunk2");

    job_details.stacks_a = aether_charges;

    let _faery_gauge: i32 = i32::from_str_radix(&chunks[3], 16).expect("Failed to convert chunk3");
}

pub fn determine_next_gcd(
    job_details: &mut JobDetails,
    spell_cast_type: SpellcastType,
) -> (&'static str, bool) {
    let mut next_spell_name = "";

    let mut aoe = false;
    // let mut alternate = old_alternate;

    let mut can_movement_be_restricted = false;
    let mut instant_cast_only = false;

    // VARIATIONS CASTERS
    // 1 : Constrained mono casts
    // 2 : Instants only - free movement
    // 3 : Constrained AOE casts. In the case of Scholar, no restriction. This is an exception
    // 4 : Classic - mono - user can interrupt cast by moving.
    // There is a question to make the charges dump related to this fourth choice.
    // It would be for chill play and do the dump only then, as to not make a mess in managed mode

    match spell_cast_type {
        SpellcastType::First => can_movement_be_restricted = true,
        SpellcastType::Second => instant_cast_only = true,
        SpellcastType::Third => aoe = true,
        SpellcastType::Fourth => (),
    }

    // Manage stop dot and no Dot in AOE
    let dot_allowed = !aoe && !spell::is_status_active_now(job_details, "Do Not Dot");

    if next_spell_name == "" {
        if aoe {
            next_spell_name = "Art of War"
        } else {
            // reset dot end of burst
            if should_reset_dot_end_of_burst(job_details) {
                next_spell_name = "Biolysis";
            // Reset Dot on time
            } else if dot_allowed && !spell::is_status_active_in(job_details, "Dot", 3000) {
                next_spell_name = "Biolysis";
            // Default GCD
            } else {
                if instant_cast_only {
                    next_spell_name = "Ruin II"
                } else {
                    next_spell_name = "Broil IV";
                }
            }
        }
    }

    // if spell by that name has a cast time ->
    let static_cast_needed = false; // fixme
    let movement_freedom = can_movement_be_restricted && static_cast_needed;

    return (next_spell_name, movement_freedom);
}

fn determine_next_ogcd(
    job_details: &mut JobDetails,
    _spell_cast_type: SpellcastType,
) -> &'static str {
    const MAX_WEAVE_NUMBER: i32 = 1;

    let mut next_spell_name = "";
    let burst_allowed = job_details.burst_allowed;

    // let mut alternate = old_alternate;

    // Manage stop dot and no Dot in AOE
    if (job_details.weaves < MAX_WEAVE_NUMBER) && spell::ogcd_end_before_next_gcd(job_details) {
        // --- A Priority: no losing charges Refresh charges with Aetherflow
        if should_aetherflow(job_details) {
            next_spell_name = "Aetherflow";
            // -- B priority - no drifting raid buffs
        } else if spell::is_spell_available_now(job_details, "Chain Stratagem") && burst_allowed {
            next_spell_name = "Chain Stratagem";
        // Impact imminient is delayed because doing aetherflow is a priority - Their might be a reason
        } else if spell::is_status_active_now(job_details, "Impact Imminent")
            && !spell::is_spell_available_now(job_details, "Aetherflow")
        {
            next_spell_name = "Baneful Impaction";
        } else if should_dump_aether_charges(job_details) {
            // Drain dump before using Aethercharge
            next_spell_name = "Energy Drain";
        }
    }

    return next_spell_name;
}
