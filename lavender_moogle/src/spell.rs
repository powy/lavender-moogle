use crate::job_management;
use crate::types::{CharacterDetails, JobDetails, SpellDetails};
use chrono::{DateTime, Utc};
use log::{debug, error};

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
            spell_details.unavailable_before = calculate_future_time(spell_details.cooldown, 0);
            debug!(
                "Apply CD for ability: {}, next avaibility {}",
                spell_details.name, spell_details.unavailable_before
            );
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
    if let Some(spell_details) = job_details.get_spell(spell_name) {
        let duration = spell_details.duration;
        spell_details.active_until = calculate_future_time(duration, margin_time);
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
    if let Some(spell_details) = job_details.get_spell(spell_name) {
        spell_details.active_until = calculate_future_time(duration, margin_time);
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
        return spell_details.unavailable_before < next_gcd_time;
    }
    false
}

pub fn is_spell_available_now(job_details: &mut JobDetails, spell_name: &str) -> bool {
    if let Some(spell_details) = job_details.get_spell(spell_name) {
        return spell_details.unavailable_before < Utc::now();
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

// warning - hard coded value for oGCD clipping
pub fn ogcd_end_before_next_gcd(job_details: &mut JobDetails) -> bool {
    let ogcd_lock_duration = chrono::Duration::milliseconds(1250);
    let next_gcd_time = get_next_gcd_time(job_details);
    let expected_end_time_of_ogcd = Utc::now() + ogcd_lock_duration;
    return expected_end_time_of_ogcd < next_gcd_time;
}

pub fn spell_will_be_available_soon(
    job_details: &mut JobDetails,
    spell_name: &str,
    max_waiting_duration: i32,
) -> bool {
    let max_waiting_time = Utc::now() + chrono::Duration::milliseconds(max_waiting_duration.into());
    if let Some(spell_details) = job_details.get_spell(spell_name) {
        return spell_details.unavailable_before < max_waiting_time;
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
    if let Some(gcd_desciption) = job_details.get_spell("gcd unavailable") {
        let mut next_gcd_time = gcd_desciption.active_until;

        // Replace with current time if in the past
        let now = Utc::now();
        if next_gcd_time < now {
            next_gcd_time = now;
        }

        let gcd_duration =
            chrono::Duration::milliseconds(((nb_gcd - 1) * gcd_desciption.duration).into());
        let several_gcd_time = next_gcd_time + gcd_duration;

        if let Some(spell_details) = job_details.get_spell(spell_name) {
            return spell_details.unavailable_before < several_gcd_time;
        }
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

pub fn disable_oldest_active_status(
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
