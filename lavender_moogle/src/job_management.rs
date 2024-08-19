// job_management.rs
// save current job and apply the right rotation
use std::sync::{Mutex, Arc};
use lazy_static::lazy_static;
use log::debug;
use crate::types::{CharacterDetails, reset_character_details_timestamps};
use crate::rotation_paladin;
use crate::rotation_gunbreaker;

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
        reset_character_details_timestamps(character_details);
    }
}

pub fn apply_current_job_rotation(character_details: &mut CharacterDetails) {
    let current_job_string = get_current_job();
    let current_job_str = current_job_string.as_str();
    match current_job_str {
        "PLD" => rotation_paladin::update_next_spellcast(character_details),
        "GNB" => rotation_gunbreaker::update_next_spellcast(character_details),
        _ => debug!("Unmanaged job {}", current_job_str),
    }
}
