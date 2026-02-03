use crate::job_management;
use crate::types::{CharacterDetails, JobDetails, SpellDetails};
use chrono::{DateTime, Utc};
use log::{debug, error};

// Buffs that persist through character death
const BUFFS_PERSISTING_ON_DEATH: &[&str] = &[
    "gcd unavailable",
    "Standard Step on CD",
    "Technical Step on CD",
];

// Apply the new buff to the character
pub fn apply_new_buff(character_details: &mut CharacterDetails, job_name: &str, buff_name: &str) {
    if let Some(job_details) = character_details.get_mut(job_name) {
        debug!("Directly applying new buff: {}", buff_name);
        activate_status_default_time(job_details, buff_name, 0);
    }
}

// Apply the affect of casting a spell - CD, new effects etc.
pub fn apply_spell_cast(
    character_details: &mut CharacterDetails,
    job_name: &str,
    spell_name: &str,
) {
    if let Some(job_details) = character_details.get_mut(job_name) {
        if let Some(spell_details) = job_details.get_spell(spell_name).cloned() {
            debug!("Action for ability: {}", spell_name);

            // Update last_cast_time timestamp and spell name
            job_details.last_cast_spell = spell_name.to_string();
            if let Some(spell_details_mut) = job_details.get_spell(spell_name) {
                spell_details_mut.last_cast_time = Utc::now();
            }

            // Apply self CD -- require mutable
            apply_spell_cooldown(job_details, spell_name);
            apply_gcd_on_cast(job_details, &spell_details);
            // Important to remove status before activating them !
            deactivate_related_status_on_cast(job_details, &spell_details);
            apply_new_status_on_cast(job_details, &spell_details);
            apply_ressources_on_cast(job_details, &spell_details);
            job_management::apply_spell_additional_effects(job_name, spell_name, job_details);
        } else {
            if spell_name != "attack" && spell_name != "Shot" {
                debug!("Spell {} not found for job {}", spell_name, job_name);
            }
        }
    } else {
        error!("Job {} not found in spell book", job_name);
    }
}

pub fn apply_spell_cooldown(job_details: &mut JobDetails, spell_name: &str) {
    if let Some(spell_details) = job_details.get_spell(spell_name) {
        if spell_details.cooldown > 0 {
            if spell_details.max_charges > 0 {
                // CHARGE SYSTEM: Add recovery timestamp to queue
                let recovery_time = calculate_future_time(spell_details.cooldown, 0);
                spell_details.charge_timers.push(recovery_time);
                spell_details.charge_timers.sort(); // Keep oldest first

                debug!(
                    "Apply charge CD for {}: {} charges on cooldown",
                    spell_details.name,
                    spell_details.charge_timers.len()
                );
            } else {
                // LEGACY SYSTEM: Use unavailable_before (unchanged)
                spell_details.unavailable_before = calculate_future_time(spell_details.cooldown, 0);
                debug!(
                    "Apply CD for ability: {}, next avaibility {}",
                    spell_details.name, spell_details.unavailable_before
                );
            }
        }
    } else {
        error!("Spell {} not found", spell_name);
        return;
    }
}

pub fn apply_gcd_on_cast(job_details: &mut JobDetails, spell_details: &SpellDetails) {
    if spell_details.spell_type == "GCD" {
        if spell_details.cooldown > 0 {
            activate_status_until(job_details, "gcd unavailable", spell_details.cooldown, 0);
        } else {
            activate_status_default_time(job_details, "gcd unavailable", 0);
        }
        // reset weaves
        job_details.weaves = 0;
    } else {
        job_details.weaves += 1;
    }
}

pub fn activate_status_default_time(
    job_details: &mut JobDetails,
    spell_name: &str,
    margin_time: i32,
) {
    job_details.last_activation_spell = spell_name.to_string();
    if let Some(spell_details) = job_details.get_spell(spell_name) {
        let duration = spell_details.duration;
        spell_details.active_until = calculate_future_time(duration, margin_time);
        spell_details.last_activation_time = Utc::now();
        debug!(
            "Status {} activated for {} milliseconds, until {}",
            spell_name, duration, spell_details.active_until
        );
    } else {
        error!("Effect to activate {} not found in spell book", spell_name);
    }
}

pub fn activate_status_until(
    job_details: &mut JobDetails,
    spell_name: &str,
    duration: i32,
    margin_time: i32,
) {
    job_details.last_activation_spell = spell_name.to_string();
    if let Some(spell_details) = job_details.get_spell(spell_name) {
        spell_details.active_until = calculate_future_time(duration, margin_time);
        spell_details.last_activation_time = Utc::now();
        debug!(
            "Status {} activated for {} milliseconds, until {}",
            spell_name, duration, spell_details.active_until
        );
    } else {
        error!("Effect to activate {} not found in spell book", spell_name);
    }
}

pub fn deactivate_status(job_details: &mut JobDetails, spell_name: &str) {
    if let Some(spell_details) = job_details.get_spell(spell_name) {
        spell_details.active_until = calculate_future_time(-1000, 0);
        debug!("Status {} deactivated", spell_name);
    } else {
        error!(
            "Effect to deactivate {} not found in spell book",
            spell_name
        );
    }
}

pub fn apply_new_status_on_cast(job_details: &mut JobDetails, spell_details: &SpellDetails) {
    for status_name in &spell_details.give_status {
        activate_status_default_time(job_details, status_name, 0);
    }
}

pub fn deactivate_related_status_on_cast(
    job_details: &mut JobDetails,
    spell_details: &SpellDetails,
) {
    for status_name in &spell_details.remove_status {
        deactivate_status(job_details, status_name);
    }
}

pub fn calculate_future_time(duration_value: i32, margin_time: i32) -> DateTime<Utc> {
    let current_time = Utc::now();
    let duration = chrono::Duration::milliseconds(duration_value.into());
    let margin = chrono::Duration::milliseconds(margin_time.into());
    let future_time = current_time + duration + margin;
    return future_time;
}

pub fn apply_ressources_on_cast(job_details: &mut JobDetails, spell_details: &SpellDetails) {
    if spell_details.stacks_a != 0 {
        debug!(
            "Applying {} stacks of A - old {} - new {}",
            spell_details.stacks_a,
            job_details.stacks_a,
            job_details.stacks_a + spell_details.stacks_a
        );
    }
    if spell_details.tertiary_bar != 0 {
        debug!(
            "Applying {} tertiary bar - old {} - new {}",
            spell_details.tertiary_bar,
            job_details.tertiary_bar,
            job_details.tertiary_bar + spell_details.tertiary_bar
        );
    }
    if spell_details.secondary_bar != 0 {
        debug!(
            "Applying {} secondary bar - old {} - new {}",
            spell_details.secondary_bar,
            job_details.secondary_bar,
            job_details.secondary_bar + spell_details.secondary_bar
        );
    }
    if spell_details.stacks_b != 0 {
        debug!(
            "Applying {} stacks of B - old {} - new {}",
            spell_details.stacks_b,
            job_details.stacks_b,
            job_details.stacks_b + spell_details.stacks_b
        );
    }
    job_details.mana += spell_details.mana;
    job_details.secondary_bar += spell_details.secondary_bar;
    job_details.tertiary_bar += spell_details.tertiary_bar;
    job_details.stacks_a += spell_details.stacks_a;
    job_details.stacks_b += spell_details.stacks_b;
}

pub fn get_next_gcd_time(job_details: &mut JobDetails) -> DateTime<Utc> {
    if let Some(gcd_desciption) = job_details.get_spell("gcd unavailable") {
        return gcd_desciption.active_until;
    } else {
        return Utc::now();
    }
}

pub fn is_spell_available_next_gcd(job_details: &mut JobDetails, spell_name: &str) -> bool {
    let next_gcd_time = get_next_gcd_time(job_details);
    if let Some(spell_details) = job_details.get_spell(spell_name) {
        if spell_details.max_charges > 0 {
            // CHARGE SYSTEM: Check if at least 1 charge available at next GCD
            return get_charges_available_at_time(job_details, spell_name, next_gcd_time) > 0;
        } else {
            // LEGACY SYSTEM
            return spell_details.unavailable_before < next_gcd_time;
        }
    }
    false
}

pub fn is_spell_available_now(job_details: &mut JobDetails, spell_name: &str) -> bool {
    if let Some(spell_details) = job_details.get_spell(spell_name) {
        if spell_details.max_charges > 0 {
            // CHARGE SYSTEM: Check if at least 1 charge available
            return get_charges_available_now(job_details, spell_name) > 0;
        } else {
            // LEGACY SYSTEM: Check unavailable_before
            return spell_details.unavailable_before < Utc::now();
        }
    }
    false
}

pub fn is_status_active_now(job_details: &mut JobDetails, spell_name: &str) -> bool {
    if let Some(spell_details) = job_details.get_spell(spell_name) {
        return spell_details.active_until > Utc::now();
    }
    false
}

pub fn is_status_active_next_gcd(job_details: &mut JobDetails, spell_name: &str) -> bool {
    if is_status_active_now(job_details, spell_name) {
        let next_gcd_time = get_next_gcd_time(job_details);
        if let Some(spell_details) = job_details.get_spell(spell_name) {
            let is_active_at_scheduled_gcd = spell_details.active_until > next_gcd_time;
            // WARNING if GCD is passed, doesn't work well
            return is_active_at_scheduled_gcd;
        }
    }
    false
}

/// Returns true if status is active AND has been active for _at least_ X GCDs
///
/// # Parameters
/// - `job_details`: Mutable reference to job details
/// - `status_name`: Name of the status to check
/// - `x_gcd`: Number of GCDs the status must have been active for
///
/// # Returns
/// True if status is active and has been active for >= x_gcd GCD durations
pub fn is_status_active_since_x_gcd(
    job_details: &mut JobDetails,
    status_name: &str,
    x_gcd: i32,
) -> bool {
    // Validate input
    if x_gcd < 0 {
        return false;
    }

    // Check if status is currently active
    if !is_status_active_now(job_details, status_name) {
        return false;
    }

    // Get GCD duration from "gcd unavailable" spell
    let gcd_duration_ms = if let Some(gcd_spell) = job_details.get_spell("gcd unavailable") {
        if gcd_spell.duration <= 0 {
            return false;
        }
        gcd_spell.duration
    } else {
        return false;
    };

    // Get last_activation_time timestamp
    let last_activation_time = if let Some(status_details) = job_details.get_spell(status_name) {
        status_details.last_activation_time
    } else {
        return false;
    };

    // Calculate duration status has been active
    let now = Utc::now();

    // Safety: if last_activation_time is in future, treat as just activated
    if last_activation_time > now {
        return false;
    }

    let time_active = now.signed_duration_since(last_activation_time);

    // Calculate required duration
    let required_duration_ms = x_gcd * gcd_duration_ms;
    let required_duration = chrono::Duration::milliseconds(required_duration_ms.into());

    // Return true if status has been active long enough
    time_active >= required_duration
}

// warning - hard coded value for oGCD clipping
pub fn ogcd_end_before_next_gcd(job_details: &mut JobDetails) -> bool {
    // let ogcd_lock_duration = chrono::Duration::milliseconds(1250);
    let ogcd_lock_duration = chrono::Duration::milliseconds(750);
    let next_gcd_time = get_next_gcd_time(job_details);
    let expected_end_time_of_ogcd = Utc::now() + ogcd_lock_duration;
    return expected_end_time_of_ogcd < next_gcd_time;
}

pub fn in_late_ogcd_timing(job_details: &mut JobDetails) -> bool {
    let ogcd_lock_duration = chrono::Duration::milliseconds(600);
    let late_ogcd_timing = chrono::Duration::milliseconds(1200);
    let next_gcd_time = get_next_gcd_time(job_details);
    let expected_end_time_of_ogcd = Utc::now() + ogcd_lock_duration;
    let ogcd_end_before_next_gcd = expected_end_time_of_ogcd < next_gcd_time;
    let is_close_enough_to_next_gcd = Utc::now() + late_ogcd_timing > next_gcd_time;
    return is_close_enough_to_next_gcd && ogcd_end_before_next_gcd;
}

pub fn spell_will_be_available_soon(
    job_details: &mut JobDetails,
    spell_name: &str,
    max_waiting_duration: i32,
) -> bool {
    let max_waiting_time = Utc::now() + chrono::Duration::milliseconds(max_waiting_duration.into());
    if let Some(spell_details) = job_details.get_spell(spell_name) {
        if spell_details.max_charges > 0 {
            // CHARGE SYSTEM
            return get_charges_available_at_time(job_details, spell_name, max_waiting_time) > 0;
        } else {
            // LEGACY SYSTEM
            return spell_details.unavailable_before < max_waiting_time;
        }
    }
    false
}

pub fn is_status_still_active_in_two_gcd(job_details: &mut JobDetails, spell_name: &str) -> bool {
    if is_status_active_now(job_details, spell_name) {
        if let Some(gcd_desciption) = job_details.get_spell("gcd unavailable") {
            let next_gcd_time = gcd_desciption.active_until;
            let gcd_duration = chrono::Duration::milliseconds(gcd_desciption.duration.into());
            let two_gcd_time = next_gcd_time + gcd_duration;
            if let Some(spell_details) = job_details.get_spell(spell_name) {
                // Two tests in case GCD is passed
                let is_active_at_scheduled_2_gcd = spell_details.active_until > two_gcd_time;
                let one_gcd_from_now = Utc::now() + gcd_duration;
                let is_active_in_one_gcd_duration = spell_details.active_until > one_gcd_from_now;
                return is_active_at_scheduled_2_gcd && is_active_in_one_gcd_duration;
            }
        }
    }
    false
}

// 1 GCD = next GCD. 2 GCD = the one after etc.
// Say if the spell is available at the exact time we go from one gcd to the next one
pub fn is_spell_available_in_x_gcd(
    job_details: &mut JobDetails,
    spell_name: &str,
    nb_gcd: i32,
) -> bool {
    // Check max_charges first to determine which system to use
    let max_charges = if let Some(spell_details) = job_details.get_spell(spell_name) {
        spell_details.max_charges
    } else {
        return false;
    };

    if max_charges > 0 {
        // CHARGE SYSTEM: Use charge-aware function
        return get_charges_available_in_x_gcd(job_details, spell_name, nb_gcd) > 0;
    } else {
        // LEGACY SYSTEM (unchanged)
        if let Some(gcd_desciption) = job_details.get_spell("gcd unavailable") {
            let mut next_gcd_time = gcd_desciption.active_until;
            let gcd_duration_value = gcd_desciption.duration;

            // Replace with current time if in the past
            let now = Utc::now();
            if next_gcd_time < now {
                next_gcd_time = now;
            }

            let gcd_duration =
                chrono::Duration::milliseconds(((nb_gcd - 1) * gcd_duration_value).into());
            let several_gcd_time = next_gcd_time + gcd_duration;

            if let Some(spell_details) = job_details.get_spell(spell_name) {
                return spell_details.unavailable_before < several_gcd_time;
            }
        }
    }
    false
}

/// Returns the number of charges available RIGHT NOW
/// For legacy spells (max_charges=0), returns 1 if available, 0 if on cooldown
pub fn get_charges_available_now(job_details: &mut JobDetails, spell_name: &str) -> i32 {
    if let Some(spell_details) = job_details.get_spell(spell_name) {
        if spell_details.max_charges > 0 {
            // CHARGE SYSTEM: Count charges not on cooldown
            let now = Utc::now();
            let charges_on_cooldown = spell_details
                .charge_timers
                .iter()
                .filter(|&&time| time > now)
                .count() as i32;

            return spell_details.max_charges - charges_on_cooldown;
        } else {
            // LEGACY SYSTEM: Return 1 or 0
            return if spell_details.unavailable_before < Utc::now() {
                1
            } else {
                0
            };
        }
    }
    0
}

/// Returns the number of charges available at a specific time
pub fn get_charges_available_at_time(
    job_details: &mut JobDetails,
    spell_name: &str,
    time: DateTime<Utc>,
) -> i32 {
    if let Some(spell_details) = job_details.get_spell(spell_name) {
        if spell_details.max_charges > 0 {
            // CHARGE SYSTEM: Count charges that will be ready by target time
            let charges_still_on_cooldown = spell_details
                .charge_timers
                .iter()
                .filter(|&&recovery_time| recovery_time > time)
                .count() as i32;

            return spell_details.max_charges - charges_still_on_cooldown;
        } else {
            // LEGACY SYSTEM: Return 1 or 0
            return if spell_details.unavailable_before < time {
                1
            } else {
                0
            };
        }
    }
    0
}

/// Returns the number of charges available in X GCDs
pub fn get_charges_available_in_x_gcd(
    job_details: &mut JobDetails,
    spell_name: &str,
    nb_gcd: i32,
) -> i32 {
    if let Some(gcd_description) = job_details.get_spell("gcd unavailable") {
        let mut next_gcd_time = gcd_description.active_until;

        // Replace with current time if in the past
        let now = Utc::now();
        if next_gcd_time < now {
            next_gcd_time = now;
        }

        let gcd_duration =
            chrono::Duration::milliseconds(((nb_gcd - 1) * gcd_description.duration).into());
        let target_time = next_gcd_time + gcd_duration;

        return get_charges_available_at_time(job_details, spell_name, target_time);
    }
    0
}

/// Returns true if the spell will reach max charges before target time
/// This is critical for charge overcapping detection
pub fn will_overcap_before_time(
    job_details: &mut JobDetails,
    spell_name: &str,
    time: DateTime<Utc>,
) -> bool {
    // Extract max_charges first to avoid multiple borrows
    let max_charges = if let Some(spell_details) = job_details.get_spell(spell_name) {
        spell_details.max_charges
    } else {
        return false;
    };

    if max_charges > 0 {
        // Check if any charge timer will expire before target time
        // AND we're not already at max charges
        let current_charges = get_charges_available_now(job_details, spell_name);

        if current_charges >= max_charges {
            return true; // Already overcapped
        }

        // Check if recovery would cause overcap
        let charges_at_time = get_charges_available_at_time(job_details, spell_name, time);
        return charges_at_time >= max_charges;
    } else {
        // LEGACY SYSTEM: Not applicable
        return false;
    }
}

/// Returns true if the spell will hit max charges in X GCDs
pub fn will_overcap_in_x_gcd(job_details: &mut JobDetails, spell_name: &str, nb_gcd: i32) -> bool {
    if let Some(gcd_description) = job_details.get_spell("gcd unavailable") {
        let mut next_gcd_time = gcd_description.active_until;

        let now = Utc::now();
        if next_gcd_time < now {
            next_gcd_time = now;
        }

        let gcd_duration =
            chrono::Duration::milliseconds(((nb_gcd - 1) * gcd_description.duration).into());
        let target_time = next_gcd_time + gcd_duration;

        return will_overcap_before_time(job_details, spell_name, target_time);
    }
    false
}

pub fn is_status_expiring(job_details: &mut JobDetails, status_name: &str) -> bool {
    let status_active = is_status_active_now(job_details, status_name);
    let is_status_still_active_in_two_gcd =
        is_status_still_active_in_two_gcd(job_details, status_name);
    return status_active && !is_status_still_active_in_two_gcd;
}

pub fn is_status_active_in(job_details: &mut JobDetails, spell_name: &str, duration: i32) -> bool {
    let calculated_time = Utc::now() + chrono::Duration::milliseconds(duration.into());
    if let Some(spell_details) = job_details.get_spell(spell_name) {
        return spell_details.active_until > calculated_time;
    }
    return false;
}

pub fn is_status_finishing_before(
    job_details: &mut JobDetails,
    spell_name_1: &str,
    spell_name_2: &str,
) -> bool {
    let spell_1_end: DateTime<Utc>;
    let spell_2_end: DateTime<Utc>;
    if let Some(spell_details) = job_details.get_spell(spell_name_1) {
        spell_1_end = spell_details.active_until;
    } else {
        return false;
    }

    if let Some(spell_details) = job_details.get_spell(spell_name_2) {
        spell_2_end = spell_details.active_until;
    } else {
        return false;
    }

    return spell_1_end < spell_2_end;
}

pub fn disable_oldest_of_2_active_status(
    job_details: &mut JobDetails,
    spell_name_1: &str,
    spell_name_2: &str,
) {
    let spell_1_active: bool = is_status_active_now(job_details, spell_name_1);
    let spell_2_active: bool = is_status_active_now(job_details, spell_name_2);
    if spell_1_active && !spell_2_active {
        deactivate_status(job_details, spell_name_1);
    } else if !spell_1_active && spell_2_active {
        deactivate_status(job_details, spell_name_2);
    } else if spell_1_active && spell_2_active {
        if is_status_finishing_before(job_details, spell_name_1, spell_name_2) {
            deactivate_status(job_details, spell_name_1);
        } else {
            deactivate_status(job_details, spell_name_2);
        }
    }
}

pub fn _disable_oldest_of_3_active_status(
    job_details: &mut JobDetails,
    spell_name_1: &str,
    spell_name_2: &str,
    spell_name_3: &str,
) {
    let spell_1_active = is_status_active_now(job_details, spell_name_1);
    let spell_2_active = is_status_active_now(job_details, spell_name_2);
    let spell_3_active = is_status_active_now(job_details, spell_name_3);

    let mut active_spells = vec![];

    if spell_1_active {
        active_spells.push(spell_name_1);
    }
    if spell_2_active {
        active_spells.push(spell_name_2);
    }
    if spell_3_active {
        active_spells.push(spell_name_3);
    }

    if active_spells.len() <= 1 {
        // Nothing to deactivate if 0 or only 1 is active
        return;
    }

    // Find the one that finishes the earliest
    let mut oldest = active_spells[0];
    for &spell in &active_spells[1..] {
        if is_status_finishing_before(job_details, spell, oldest) {
            oldest = spell;
        }
    }

    deactivate_status(job_details, oldest);
}

pub fn burst_allowed(job_details: &mut JobDetails) -> bool {
    return !is_status_active_now(job_details, "Delayed Burst");
}

/// Handles character death by clearing all buffs while preserving cooldowns.
///
/// This function:
/// - Clears all active_until timestamps (buffs) except those in BUFFS_PERSISTING_ON_DEATH
/// - Preserves all unavailable_before timestamps (cooldowns)
/// - Resets all resource gauges to 0
pub fn handle_character_death(character_details: &mut CharacterDetails) {
    let now_minus_500ms = Utc::now() - chrono::Duration::milliseconds(500);

    for job_details in character_details.values_mut() {
        // Reset all resources to zero
        job_details.mana = 0;
        job_details.secondary_bar = 0;
        job_details.tertiary_bar = 0;
        job_details.stacks_a = 0;
        job_details.stacks_b = 0;
        job_details.weaves = 0;

        // Clear all buffs (active_until) EXCEPT those in BUFFS_PERSISTING_ON_DEATH
        for spell in job_details.spells.values_mut() {
            if !BUFFS_PERSISTING_ON_DEATH.contains(&spell.name.as_str()) {
                spell.active_until = now_minus_500ms;
                spell.last_activation_time = now_minus_500ms;
            }
            // IMPORTANT: Preserve cooldowns (FFXIV behavior)
            // Do NOT touch unavailable_before
            // Do NOT touch charge_timers
            // Do NOT touch last_cast_time - spells were still cast before death
        }
    }
}

/// Activate a status on all jobs (for generic commands like delay burst)
pub fn activate_status_for_any_job(character_details: &mut CharacterDetails, status_name: &str) {
    for job_details in character_details.values_mut() {
        if job_details.spells.contains_key(status_name) {
            activate_status_default_time(job_details, status_name, 0);
        }
    }
}

/// Deactivate a status on all jobs
pub fn deactivate_status_for_any_job(character_details: &mut CharacterDetails, status_name: &str) {
    for job_details in character_details.values_mut() {
        if job_details.spells.contains_key(status_name) {
            deactivate_status(job_details, status_name);
        }
    }
}
