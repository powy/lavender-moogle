// rotation_summoner.rs
// Rotation logic for summoner job

use log::error;

use crate::memory;
use crate::spell;
use crate::types::{CharacterDetails, JobDetails};

pub fn update_next_spellcast(character_details: &mut CharacterDetails, aoe: bool, alternate: bool) {
    if let Some(job_details) = character_details.get_mut("SMN") {
        // update status

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

        // check oGCD
        let mut next_spell_name = "";

        // No clipping?

        // FIXME - accept some small clipping for the GNB reactional skills
        if spell::ogcd_end_before_next_gcd(job_details) {
            // --- A Priority: no drifting raid buffs
            if spell::is_spell_available_now(job_details, "Searing Light") {
                next_spell_name = "Searing Light";

            // --- Aerth charges
            } else if spell::is_status_active_now(job_details, "Searing Light Buff")
                && job_details.stacks_a > 0
            {
                next_spell_name = "Necrotize";
            } else if spell::is_spell_available_now(job_details, "Energy Drain")
                && job_details.stacks_a <= 0
            {
                next_spell_name = "Energy Drain";

            // -- Summons special actions
            } else if spell::is_status_active_now(job_details, "Mountain Buster Ready") {
                next_spell_name = "Mountain Buster";
            } else if spell::is_status_active_now(job_details, "Ekindle Ready") {
                next_spell_name = "Enkindle Bahamut";
            } else if should_flare(job_details) {
                next_spell_name = "Deathflare";
            } else if spell::is_status_active_now(job_details, "Ruby's Glimmer") {
                next_spell_name = "Searing Flash";
            } else if should_swiftcast(job_details) {
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
                next_spell_name = "Ruin III";

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
            } else if job_details.stacks_b > 0 {
                next_spell_name = "Gemshine";

            // hard-casted slipstream
            } else if spell::is_status_active_next_gcd(job_details, "Slipstream Ready") {
                next_spell_name = "Slipstream";
                // FIXME FIXME BROKEN
            } else if spell::is_status_active_next_gcd(job_details, "Further Ruin") {
                next_spell_name = "Ruin IV";
            } else {
                next_spell_name = "Ruin III";
            }
        }
        if next_spell_name != "" {
            memory::communicate_next_spellcast(job_details, next_spell_name, aoe, alternate);
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

fn should_swiftcast(job_details: &mut JobDetails) -> bool {
    let slipstream_ready = spell::is_status_active_next_gcd(job_details, "Slipstream Ready");
    let garuda_active = spell::is_status_active_next_gcd(job_details, "Garuda");
    let swiftcast_available = spell::is_spell_available_now(job_details, "Swiftcast");
    return slipstream_ready && garuda_active && swiftcast_available;
    // return false;
}

fn should_swiftcast_splitstream(job_details: &mut JobDetails) -> bool {
    let slipstream_ready = spell::is_status_active_next_gcd(job_details, "Slipstream Ready");
    let garuda_active = spell::is_status_active_next_gcd(job_details, "Garuda");
    let swiftcast_active = spell::is_status_active_next_gcd(job_details, "Swiftcast Buff");

    return slipstream_ready && garuda_active && swiftcast_active;
    // return false
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
