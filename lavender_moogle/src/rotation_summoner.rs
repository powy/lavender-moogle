// rotation_summoner.rs
// Rotation logic for summoner job

use log::error;

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
            memory::communicate_next_spellcast(job_details, spell_cast_type, next_spell_name, true);
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
}

fn should_swiftcast_splitstream(job_details: &mut JobDetails) -> bool {
    let slipstream_ready = spell::is_status_active_next_gcd(job_details, "Slipstream Ready");
    let garuda_active = spell::is_status_active_next_gcd(job_details, "Garuda");
    let swiftcast_active = spell::is_status_active_next_gcd(job_details, "Swiftcast Buff");

    return slipstream_ready && garuda_active && swiftcast_active;
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

// The Summoner job gauge is not decoded (job_management has no SMN arm).
// Layout of the gauge bytes (stance = byte 0x07):
// 0x00 trance milliseconds (u16)
// 0x02 attunement milliseconds (u16)
// 0x04 aetherflow stacks (Arcanist)
// 0x06 attunement (Garuda attunement is half a byte)
// 0x07 stance: bits 0-1 aetherflow stacks (Summoner),
//      bits 2-3 active primal (0x4 Ifrit, 0x8 Titan, 0xC Garuda),
//      bit 4 next summon (0 Bahamut, 1 Phoenix),
//      bits 5-7 usable arcanum (0x20 Ruby, 0x40 Topaz, 0x80 Emerald)
