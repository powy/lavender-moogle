// rotation_dancer.rs
// Rotation logic for dancer job

use log::{debug, error};

use crate::memory;
use crate::spell;
use crate::types::{CharacterDetails, JobDetails};

pub fn update_next_spellcast(character_details: &mut CharacterDetails) {
    if let Some(job_details) = character_details.get_mut("DNC") {
        // update status
        // check oGCD
        let mut next_spell_name = "";

        // update stack min / max

        // Fans
        if job_details.stacks_a > 4 {
            job_details.stacks_a = 4;
        } else if job_details.stacks_a < 0 {
            job_details.stacks_a = 0;
        }

        // Dancing
        if spell::is_status_active_now(job_details, "Standard Dancing") {
            next_spell_name = get_next_step(job_details, "Standard Finish");
        } else if spell::is_status_active_now(job_details, "Technical Dancing") {
            next_spell_name = get_next_step(job_details, "Technical Finish");
        }

        // oGCD - No clipping?
        if next_spell_name == "" && spell::ogcd_end_before_next_gcd(job_details) {
            // In Burst
            if spell::is_status_active_now(job_details, "Devilement Buff") {
                next_spell_name = "";
            } else
            // Classical no buff

            // Dance 3 proc
            if spell::is_status_active_now(job_details, "Threefold Fan Dance") {
                next_spell_name = "Fan Dance III";
            } else
            // Flourish out of buff
            if !spell::spell_will_be_available_soon(job_details, "Devilement", 45000) // Not before burst
                && !spell::is_status_active_now(job_details, "Silken Symmetry")
                && !spell::is_status_active_now(job_details, "Silken Flow")
            {
                next_spell_name = "Flourish";
            } else
            // No overcap on Fans
            if job_details.stacks_a >= 4
                && (spell::is_status_active_now(job_details, "Silken Symmetry")
                    || spell::is_status_active_now(job_details, "Silken Flow"))
            {
                next_spell_name = "Fan Dance";
            }
        }

        // No oGCD selected, prepare next GCD

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
            // Basic combo procs
            if spell::is_status_active_next_gcd(job_details, "Silken Flow") {
                next_spell_name = "Fountainfall";
            } else if spell::is_status_active_next_gcd(job_details, "Silken Symmetry") {
                next_spell_name = "Reverse Cascade";
            } else
            // Basic Combo
            if spell::is_status_active_next_gcd(job_details, "Fountain Ready") {
                next_spell_name = "Fountain";
            } else {
                next_spell_name = "Cascade";
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

pub fn update_job_gauge(job_details: &mut JobDetails, data: &str) {
    // debug!("Job gauge update: {}", data);
    let chunks: Vec<String> = data
        .as_bytes() // Convert the string to a byte slice
        .chunks(2) // Split into chunks of 2 bytes
        .map(|chunk| String::from_utf8_lossy(chunk).to_string()) // Convert each chunk back to a String
        .collect();

    let feathers: i32 = i32::from_str_radix(&chunks[1], 16).expect("Failed to convert chunk2");
    let spirit: i32 = i32::from_str_radix(&chunks[2], 16).expect("Failed to convert chunk3");
    let step1: i32 = i32::from_str_radix(&chunks[3], 16).expect("Failed to convert chunk4");
    let step2: i32 = i32::from_str_radix(&chunks[4], 16).expect("Failed to convert chunk5");
    let step3: i32 = i32::from_str_radix(&chunks[5], 16).expect("Failed to convert chunk6");
    let step4: i32 = i32::from_str_radix(&chunks[6], 16).expect("Failed to convert chunk7");
    let current_step: i32 = i32::from_str_radix(&chunks[7], 16).expect("Failed to convert chunk8");

    job_details.secondary_bar = spirit;
    job_details.stacks_a = feathers; // Stacks_a is feathers in DNC job
    job_details.stacks_b = match current_step {
        0 => step1,
        1 => step2,
        2 => step3,
        3 => step4,
        _ => {
            0 // Default value if current_step is not 1, 2, 3, or 4
        }
    };

    debug!(
        "feathers: {}, spirit: {}, step1: {}, step2: {}, step3: {}, step4: {}, current_step: {}",
        feathers, spirit, step1, step2, step3, step4, current_step
    );
    debug!(
        "job details: bar {} feathers {} next step {}",
        job_details.secondary_bar, job_details.stacks_a, job_details.stacks_b
    );
}

// Determine the next step to cast while in a dance
fn get_next_step<'a>(job_details: &mut JobDetails, finish_spell: &'a str) -> &'a str {
    match job_details.stacks_b {
        1 => "Emboite",
        2 => "Entrechat",
        3 => "Jete",
        4 => "Pirouette",
        _ => finish_spell,
    }
}
