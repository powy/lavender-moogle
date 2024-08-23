// job_management.rs
// save current job and apply the right rotation
use std::sync::{Mutex, Arc};
use lazy_static::lazy_static;
use log::debug;
use crate::types::{JobDetails, CharacterDetails, reset_character_status};
use crate::rotation_paladin;
use crate::rotation_gunbreaker;
use crate::rotation_summoner;
use crate::rotation_scholar;

lazy_static! {
    static ref CURRENT_JOB: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

pub fn get_current_job() -> String {
    let current_job = CURRENT_JOB.lock().unwrap();
    current_job.clone()
}

pub fn update_current_job(character_details: &mut CharacterDetails, new_job: &str) {
    let mut current_job = CURRENT_JOB.lock().unwrap();
    if *current_job != new_job {
        debug!("changing job to {}", new_job);
        *current_job = new_job.to_string();
        reset_character_status(character_details);
    }
}

pub fn apply_current_job_rotation(character_details: &mut CharacterDetails) {
    let current_job_string = get_current_job();
    let job_name = current_job_string.as_str();
    match job_name {
        "PLD" => rotation_paladin::update_next_spellcast(character_details),
        "GNB" => rotation_gunbreaker::update_next_spellcast(character_details),
        "SMN" => rotation_summoner::update_next_spellcast(character_details),
        "SCH" => rotation_scholar::update_next_spellcast(character_details),
        _ => debug!("Unmanaged job {}", job_name),
    }
}


pub fn apply_spell_additional_effects(job_name: &str, spell_name: &str, job_details: &mut JobDetails) {
    match job_name {
        "PLD" => rotation_paladin::apply_spell_additional_effects(spell_name, job_details),
        "GNB" => rotation_gunbreaker::apply_spell_additional_effects(spell_name, job_details),
        "SMN" => rotation_summoner::apply_spell_additional_effects(spell_name, job_details),
        "SCH" => rotation_scholar::apply_spell_additional_effects(spell_name, job_details),
        _ => debug!("Unmanaged job {}", job_name),
    }
}
