// rotation_scholar.rs
// Rotation logic for scholar job

use log::{debug, error};

use crate::job_management;
use crate::memory;
use crate::spell;
use crate::types::CastingStage;
use crate::types::{CharacterDetails, JobDetails, SpellcastType};

pub fn update_next_spellcast(
    character_details: &mut CharacterDetails,
    spell_cast_type: SpellcastType,
    _aoe: bool,
    _alternate: bool,
) {
    if let Some(job_details) = character_details.get_mut("SCH") {
        new_communicate_next_cast_and_movement(job_details, spell_cast_type);
    } else {
        error!("JOB not found in spell book");
    }
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
) -> &'static str {
    let mut next_spell_name;

    let mut aoe = false;
    let mut instant_cast_only = false;

    // Variations:
    // First / Fourth : mono casts
    // Second : instants only
    // Third : AOE

    match spell_cast_type {
        SpellcastType::First => (),
        SpellcastType::Second => instant_cast_only = true,
        SpellcastType::Third => aoe = true,
        SpellcastType::Fourth => (),
    }

    // No dot in AOE or when the "Do Not Dot" status is active
    let dot_allowed = !aoe && !spell::is_status_active_now(job_details, "Do Not Dot");

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

    if next_spell_name == "" {
        next_spell_name = "do nothing";
    }

    return next_spell_name;
}

fn determine_next_ogcd(
    job_details: &mut JobDetails,
    _spell_cast_type: SpellcastType,
) -> &'static str {
    const MAX_WEAVE_NUMBER: i32 = 1;

    let mut next_spell_name = "";
    let burst_allowed = job_details.burst_allowed;


    if (job_details.weaves < MAX_WEAVE_NUMBER) && spell::ogcd_end_before_next_gcd(job_details) {
        // --- A Priority: no losing charges Refresh charges with Aetherflow
        if should_aetherflow(job_details) {
            next_spell_name = "Aetherflow";
            // -- B priority - no drifting raid buffs
        } else if spell::is_spell_available_now(job_details, "Chain Stratagem") && burst_allowed {
            next_spell_name = "Chain Stratagem";
        // Impact Imminent is delayed because Aetherflow is a priority
        } else if spell::is_status_active_now(job_details, "Impact Imminent")
            && !spell::is_spell_available_now(job_details, "Aetherflow")
        {
            next_spell_name = "Baneful Impaction";
        }
    }

    if next_spell_name == "" {
        next_spell_name = "do nothing";
    }

    return next_spell_name;
}

// FIXME has_movement_freedom usage and content
// Case of Casting stage GCd available : based on next gcd
// Case of Casting stage in casting : based on the currently casting spells (saved in job details)
fn has_movement_freedom(
    job_details: &mut JobDetails,
    _spell_cast_type: SpellcastType,
    spell_name: &str,
    casting_stage: CastingStage,
) -> bool {
    match casting_stage {
        // Can't restrict movement between GCDs
        CastingStage::WeaveAvailable | CastingStage::AfterWeaves => return true,
        CastingStage::InCasting | CastingStage::WeaveSoon => return false,
        CastingStage::GcdAvailable => {
            // Checking if the following spell is blocking
            // FIXME manage swiftcast etc.
            if let Some(next_spell_details) = job_details.get_spell(spell_name).cloned() {
                return next_spell_details.casting_time_ms <= 0;
            } else {
                error!("Spell {} not found for shared memory", spell_name);
            }
        }
    }
    return true;
}

fn new_communicate_next_cast_and_movement(
    job_details: &mut JobDetails,
    spell_cast_type: SpellcastType,
) {
    // FIXME : movement to be defined here not in the next cast.
    // Depends on final gcd selected, phase, and spell_cast_type.
    // make it a function actually
    let casting_stage = job_management::get_casting_stage(job_details);
    if casting_stage != job_details.current_casting_stage {
        debug!(
            "Casting stage changed: {:?} -> {:?}",
            job_details.current_casting_stage, casting_stage
        );
        job_details.current_casting_stage = casting_stage;
    }
    let next_gcd_spell_name = determine_next_gcd(job_details, spell_cast_type);
    let next_ogcd_spell_name = determine_next_ogcd(job_details, spell_cast_type);
    let movement_freedom = has_movement_freedom(
        job_details,
        spell_cast_type,
        next_gcd_spell_name,
        casting_stage,
    );

    let next_spell_name = match casting_stage {
        CastingStage::GcdAvailable | CastingStage::AfterWeaves => next_gcd_spell_name,
        CastingStage::InCasting => "",
        CastingStage::WeaveSoon | CastingStage::WeaveAvailable => next_ogcd_spell_name,
    };

    if next_spell_name != "" {
        memory::communicate_next_spellcast(
            job_details,
            spell_cast_type,
            next_spell_name,
            movement_freedom,
        );
    }
}

//FIXME MAJOR BUGS
// Being in 2nd spellcast doesnt force the movement freedom
// Restriction pop in 2nd spell cast while 100% of GCD are actually instant
// Restriction actually doesn't stop an already pressed button
// We need to overall the system at polling to see that the keypressed get canceled when passing freedom from yes to no
