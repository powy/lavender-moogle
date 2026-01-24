// rotation_dancer.rs
// Rotation logic for dancer job

use log::{debug, error};
// use log::error;

use crate::memory;
use crate::spell;
use crate::types::{CharacterDetails, JobDetails};

pub fn update_next_spellcast(character_details: &mut CharacterDetails, aoe: bool, alternate: bool) {
    if let Some(job_details) = character_details.get_mut("DNC") {
        const MAX_WEAVE_NUMBER: i32 = 2;
        // update status
        // check oGCD
        let mut next_spell_name = "";
        let burst_allowed = job_details.burst_allowed;
        let delayed_burst = !burst_allowed;

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
        if next_spell_name == ""
            && spell::is_spell_available_now(job_details, "Devilment")
            && spell::is_status_active_now(job_details, "Technical Finish Buff")
        {
            next_spell_name = "Devilment";
        }

        // Early dancing to avoid drifting
        // Start the burst Window
        // Tech step to go into burst (unless delayed_burst)
        if next_spell_name == ""
            && !delayed_burst
            && !spell::is_status_active_in(job_details, "Technical Step on CD", 1250)
            && spell::spell_will_be_available_soon(job_details, "Devilment", 7000)
        {
            next_spell_name = "Technical Step";
        }

        // Normal dance
        // HOLD stand step if 6s or less before tech step
        // Hold if finishing move available (last dance) Should happen only during burst
        // Hold if Flourish to have instant one
        if next_spell_name == "" && should_standard_step(job_details, alternate) {
            next_spell_name = "Standard Step";
        }

        // oGCD - No clipping?
        // In Burst
        if next_spell_name == ""
            && (job_details.weaves < MAX_WEAVE_NUMBER)
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
                if aoe {
                    next_spell_name = "Fan Dance II";
                } else {
                    next_spell_name = "Fan Dance";
                }
            }
        }

        // oGCD - No clipping?
        // Classical no buff
        if (next_spell_name == "")
            && (job_details.weaves < MAX_WEAVE_NUMBER)
            && spell::ogcd_end_before_next_gcd(job_details)
        {
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
            // Not before burst unless dump ressources
                && !(spell::spell_will_be_available_soon(job_details, "Devilment", 30000) && !job_details.dump_resources)
                // && !spell::is_status_active_now(job_details, "Silken Symmetry")
                // && !spell::is_status_active_now(job_details, "Silken Flow")
                // If Threefold Fan Dance, dump it before flourish for no overlap
                && !spell::is_status_active_now(job_details, "Threefold Fan Dance")
            {
                next_spell_name = "Flourish";
            } else
            // No overcap on Fans
            if (job_details.stacks_a >= 4
                && (spell::is_status_active_now(job_details, "Silken Symmetry")
                    || spell::is_status_active_now(job_details, "Silken Flow")
                    || spell::is_status_active_now(job_details, "Flourishing Symmetry")
                    || spell::is_status_active_now(job_details, "Flourishing Flow")))
                    // Dump ressources
                    || (job_details.dump_resources && job_details.stacks_a >= 1)
            {
                if aoe {
                    next_spell_name = "Fan Dance II";
                } else {
                    next_spell_name = "Fan Dance";
                }
            }
        }

        // No oGCD selected, prepare next GCD

        // FIXME - add a last GCD save on procs

        // No overcap on Esprit
        if next_spell_name == "" && job_details.secondary_bar >= 65 {
            next_spell_name = "Saber Dance";
        }

        // dump esprit if dump ressources
        if next_spell_name == "" && job_details.dump_resources && job_details.secondary_bar >= 50 {
            next_spell_name = "Saber Dance";
        }

        // check burst special treatment GCD
        // Note: wait for active Devilment before going into more big spells
        if next_spell_name == "" && spell::is_status_active_next_gcd(job_details, "Devilment Buff")
        {
            // First use the
            if spell::is_status_active_next_gcd(job_details, "Last Dance Ready") {
                next_spell_name = "Last Dance";
            } else if job_details.secondary_bar >= 50 {
                next_spell_name = "Saber Dance";
            } else if spell::is_status_active_next_gcd(job_details, "Flourishing Starfall") {
                next_spell_name = "Starfall Dance";
            } else if spell::is_status_active_next_gcd(job_details, "Flourishing Finish") {
                next_spell_name = "Tillana";
            }
        }

        // check standart GCD
        // Not in burst window - avoid overcap blood or classic combo
        if next_spell_name == "" {
            // Basic combo procs
            // Repetition is not pretty, but it is an simple way to give p^riority on procs rather than on Flourish
            // FIXME - the system doesn't manage double proc with only one eaten by an ability
            if spell::is_status_active_next_gcd(job_details, "Silken Flow") {
                if aoe {
                    next_spell_name = "Bloodshower";
                } else {
                    next_spell_name = "Fountainfall";
                }
            } else if spell::is_status_active_next_gcd(job_details, "Silken Symmetry") {
                if aoe {
                    next_spell_name = "Rising Windmill";
                } else {
                    next_spell_name = "Reverse Cascade";
                }
            } else if spell::is_status_active_next_gcd(job_details, "Flourishing Flow") {
                if aoe {
                    next_spell_name = "Bloodshower";
                } else {
                    next_spell_name = "Fountainfall";
                }
            } else if spell::is_status_active_next_gcd(job_details, "Flourishing Symmetry") {
                if aoe {
                    next_spell_name = "Rising Windmill";
                } else {
                    next_spell_name = "Reverse Cascade";
                }
            } else
            // Follow up dance
            if spell::is_status_active_next_gcd(job_details, "Last Dance Ready") {
                next_spell_name = "Last Dance";
            } else
            // Basic Combo
            if spell::is_status_active_next_gcd(job_details, "Fountain Ready") && !aoe {
                next_spell_name = "Fountain";
            } else if spell::is_status_active_next_gcd(job_details, "Shower Ready") && aoe {
                next_spell_name = "Bladeshower";
            } else if aoe {
                next_spell_name = "Windmill";
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

// Nouvelle strategie, si les 2 sonta ctifs, on erase 1 seul des deux

// We are doing a hack to simulate two buff at the same time
// on using a silken proc, the flourishing proc (if existing) is transfrmed into a silken proc
// Silken proc is ended. Becaus eof this system, silken is removed from the normal status removal system
pub fn apply_spell_additional_effects(spell_name: &str, job_details: &mut JobDetails) {
    match spell_name {
        "Bloodshower" => {
            spell::disable_oldest_of_2_active_status(job_details, "Silken Flow", "Flourishing Flow")
        }
        "Fountainfall" => {
            spell::disable_oldest_of_2_active_status(job_details, "Silken Flow", "Flourishing Flow")
        }
        "Rising Windmill" => spell::disable_oldest_of_2_active_status(
            job_details,
            "Silken Symmetry",
            "Flourishing Symmetry",
        ),
        "Reverse Cascade" => spell::disable_oldest_of_2_active_status(
            job_details,
            "Silken Symmetry",
            "Flourishing Symmetry",
        ),

        "Technical Step" => {
            job_details.burst_allowed = true;
            job_details.dump_resources = false;
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

    // // // Debug output for each chunk
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

    let feathers: i32 = i32::from_str_radix(&chunks[2], 16).expect("Failed to convert chunk2");
    // debug!("feathers 1: {}", feathers);
    let esprit: i32 = i32::from_str_radix(&chunks[3], 16).expect("Failed to convert chunk3");
    // debug!("esprit 1: {}", esprit);
    let step1: i32 = i32::from_str_radix(&chunks[4], 16).expect("Failed to convert chunk4");
    // debug!("Step 1: {}", step1);
    let step2: i32 = i32::from_str_radix(&chunks[5], 16).expect("Failed to convert chunk5");
    // debug!("Step 2: {}", step2);
    let step3: i32 = i32::from_str_radix(&chunks[6], 16).expect("Failed to convert chunk6");
    // debug!("Step 3: {}", step3);
    let step4: i32 = i32::from_str_radix(&chunks[7], 16).expect("Failed to convert chunk7");
    // debug!("Step 4: {}", step4);
    let current_step: i32 = i32::from_str_radix(&chunks[8], 16).expect("Failed to convert chunk8");
    // debug!("current_step: {}", current_step);

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
    // debug!("job_details.stacks_b: {}", job_details.stacks_b);
}

fn should_standard_step(job_details: &mut JobDetails, alternate: bool) -> bool {
    let standart_available: bool =
        !spell::is_status_active_in(job_details, "Standard Step on CD", 1250);
    // Note: If burst is not allowed, we ignore the fact that technical step is coming up
    let technical_soon: bool =
        !spell::is_status_active_in(job_details, "Technical Step on CD", 7000)
            && job_details.burst_allowed;
    let last_dance_ready: bool = spell::is_status_active_now(job_details, "Last Dance Ready");
    let flourish_soon: bool = spell::spell_will_be_available_soon(job_details, "Flourish", 5000);
    let instant_possible: bool = spell::is_status_active_now(job_details, "Finishing Move Ready");
    let devilment_running: bool = spell::is_status_active_now(job_details, "Devilment Buff");

    if standart_available {
        // delayed_burst - not delaying standard step, as we don't know when is burst
        // FIXME deprecate the whole alternate for the burst delay triggers
        if !job_details.burst_allowed || alternate {
            return true;
        }

        // We have instant proc, not in burst, use
        if instant_possible && !devilment_running {
            return true;
        }

        // We are in burst and expect something else before
        if flourish_soon || last_dance_ready {
            return false;
        }

        // Delay to not drift technical step
        // But not the case in delayed_burst
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
