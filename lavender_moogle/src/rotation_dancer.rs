// rotation_dancer.rs
// Rotation logic for dancer job

use log::error;

use crate::memory;
use crate::spell;
use crate::types::{CharacterDetails, JobDetails};

pub fn update_next_spellcast(character_details: &mut CharacterDetails, aoe: bool, alternate: bool) {
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

        // Devilment - do not avoid clipping
        // Due to tech step "special" category
        if spell::is_spell_available_now(job_details, "Devilment")
            && spell::is_status_active_now(job_details, "Technical Finish Buff")
        {
            next_spell_name = "Devilment";
        }

        // Early dancing to avoid drifting
        // Start the burst Window
        // Tech step to go into burst
        if !spell::is_status_active_in(job_details, "Technical Step on CD", 1250) {
            next_spell_name = "Technical Step";
        }

        // Normal dance
        // HOLD stand step if 6s or less before tech step
        // Hold if finishing move available (last dance) Should happen only during burst
        // Hold if Flourish to have instant one
        if next_spell_name == "" && should_standard_step(job_details) {
            next_spell_name = "Standard Step";
        }

        // oGCD - No clipping?
        // In Burst
        if next_spell_name == ""
            && spell::ogcd_end_before_next_gcd(job_details)
            && spell::is_status_active_now(job_details, "Devilment Buff")
        {
            if spell::is_spell_available_now(job_details, "Flourish")
                && !spell::is_status_active_now(job_details, "Threefold Fan Dance")
            {
                next_spell_name = "Flourish";
            } else
            // Dance 3 proc
            if spell::is_status_active_now(job_details, "Threefold Fan Dance") {
                next_spell_name = "Fan Dance III";
            } else
            // Dance 4
            if spell::is_status_active_now(job_details, "Fourfold Fan Dance") {
                next_spell_name = "Fan Dance IV";
            } else
            // Spend Dance 1
            if job_details.stacks_a >= 1 {
                next_spell_name = "Fan Dance";
            }
        }

        // oGCD - No clipping?
        // Classical no buff
        if next_spell_name == "" && spell::ogcd_end_before_next_gcd(job_details) {
            // Dance 3 proc
            if spell::is_status_active_now(job_details, "Threefold Fan Dance") {
                next_spell_name = "Fan Dance III";
            } else
            // Dance 4
            if spell::is_status_active_now(job_details, "Fourfold Fan Dance") {
                next_spell_name = "Fan Dance IV";
            } else
            // Flourish out of buff
            if spell::is_spell_available_now(job_details, "Flourish")
                && !spell::spell_will_be_available_soon(job_details, "Devilment", 30000) // Not before burst
                // && !spell::is_status_active_now(job_details, "Silken Symmetry")
                // && !spell::is_status_active_now(job_details, "Silken Flow")
                && !spell::is_status_active_now(job_details, "Threefold Fan Dance")
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

        // No overcap on Esprit
        if next_spell_name == "" && job_details.secondary_bar >= 80 {
            next_spell_name = "Saber Dance";
        }

        // check burst special treatment GCD
        // Note: wait for active Devilment before going into more big spells
        if next_spell_name == "" && spell::is_status_active_next_gcd(job_details, "Devilment Buff")
        {
            // First use the
            if spell::is_status_active_next_gcd(job_details, "Last Dance Ready") {
                next_spell_name = "Last Dance";
            } else if spell::is_status_active_next_gcd(job_details, "Flourishing Starfall") {
                next_spell_name = "Starfall Dance";
            } else if spell::is_status_active_next_gcd(job_details, "Flourishing Finish")
                && job_details.secondary_bar <= 50
            {
                next_spell_name = "Tillana";
            } else if job_details.secondary_bar >= 50 {
                next_spell_name = "Saber Dance";
            }
        }

        // check standart GCD
        // Not in burst window - avoid overcap blood or classic combo
        if next_spell_name == "" {
            // Basic combo procs
            if spell::is_status_active_next_gcd(job_details, "Silken Flow") {
                next_spell_name = "Fountainfall";
            } else if spell::is_status_active_next_gcd(job_details, "Silken Symmetry") {
                next_spell_name = "Reverse Cascade";
            } else
            // Follow up dance
            if spell::is_status_active_next_gcd(job_details, "Last Dance Ready") {
                next_spell_name = "Last Dance";
            } else
            // Basic Combo
            if spell::is_status_active_next_gcd(job_details, "Fountain Ready") {
                next_spell_name = "Fountain";
            } else {
                next_spell_name = "Cascade";
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

pub fn update_job_gauge(job_details: &mut JobDetails, data: &str) {
    // debug!("Job gauge update: {}", data);
    let chunks: Vec<String> = data
        .as_bytes() // Convert the string to a byte slice
        .chunks(2) // Split into chunks of 2 bytes
        .map(|chunk| String::from_utf8_lossy(chunk).to_string()) // Convert each chunk back to a String
        .collect();

    let feathers: i32 = i32::from_str_radix(&chunks[1], 16).expect("Failed to convert chunk2");
    let esprit: i32 = i32::from_str_radix(&chunks[2], 16).expect("Failed to convert chunk3");
    let step1: i32 = i32::from_str_radix(&chunks[3], 16).expect("Failed to convert chunk4");
    let step2: i32 = i32::from_str_radix(&chunks[4], 16).expect("Failed to convert chunk5");
    let step3: i32 = i32::from_str_radix(&chunks[5], 16).expect("Failed to convert chunk6");
    let step4: i32 = i32::from_str_radix(&chunks[6], 16).expect("Failed to convert chunk7");
    let current_step: i32 = i32::from_str_radix(&chunks[7], 16).expect("Failed to convert chunk8");

    job_details.secondary_bar = esprit;
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
}

fn should_standard_step(job_details: &mut JobDetails) -> bool {
    let standart_available: bool =
        !spell::is_status_active_in(job_details, "Standard Step on CD", 1250);
    let technical_soon: bool =
        !spell::is_status_active_in(job_details, "Technical Step on CD", 7000);
    let last_dance_ready: bool = spell::is_status_active_now(job_details, "Last Dance Ready");
    let flourish_soon: bool = spell::spell_will_be_available_soon(job_details, "Flourish", 5000);
    let instant_possible: bool = spell::is_status_active_now(job_details, "Finishing Move Ready");
    let devilment_running: bool = spell::is_status_active_now(job_details, "Devilment Buff");

    if standart_available {
        // We have instant proc, not in burst, use
        if instant_possible && !devilment_running {
            return true;
        }

        // We are in burst and expect something else before
        if flourish_soon || last_dance_ready {
            return false;
        }

        // Delay to not drift technical step
        if technical_soon {
            return false;
        }

        // normal case, use
        return true;
    }

    return false;
}

// && !spell::is_status_active_in(job_details, "Standard Step on CD", 1250)
// && spell::is_status_active_in(job_details, "Technical Step on CD", 7000)
// && !spell::is_status_active_now(job_details, "Last Dance Ready")
// && !spell::spell_will_be_available_soon(job_details, "Flourish", 6000)

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
