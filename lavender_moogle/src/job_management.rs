// job_management.rs
// save current job and apply the right rotation
use std::sync::{Mutex, Arc};
use lazy_static::lazy_static;
use log::debug;
use crate::types::{JobDetails, CharacterDetails, reset_character_status};
use crate::rotation_paladin;
use crate::rotation_dark_knight;
use crate::rotation_gunbreaker;
use crate::rotation_summoner;
use crate::rotation_scholar;
use crate::rotation_dancer;

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
        "DRK" => rotation_dark_knight::update_next_spellcast(character_details),
        "GNB" => rotation_gunbreaker::update_next_spellcast(character_details),
        "SMN" => rotation_summoner::update_next_spellcast(character_details),
        "SCH" => rotation_scholar::update_next_spellcast(character_details),
        "DNC" => rotation_dancer::update_next_spellcast(character_details),
        _ => {},
    }
}


pub fn apply_spell_additional_effects(job_name: &str, spell_name: &str, job_details: &mut JobDetails) {
    match job_name {
        "PLD" => rotation_paladin::apply_spell_additional_effects(spell_name, job_details),
        "DRK" => rotation_dark_knight::apply_spell_additional_effects(spell_name, job_details),
        "GNB" => rotation_gunbreaker::apply_spell_additional_effects(spell_name, job_details),
        "SMN" => rotation_summoner::apply_spell_additional_effects(spell_name, job_details),
        "SCH" => rotation_scholar::apply_spell_additional_effects(spell_name, job_details),
        "DNC" => rotation_dancer::apply_spell_additional_effects(spell_name, job_details),
        _ => {},
    }
}

pub fn update_job_gauge(character_details: &mut CharacterDetails, job_name: &str, raw_data0: &str, raw_data1: &str, raw_data2: &str, raw_data3: &str) {

    if let Some(job_details) = character_details.get_mut(job_name) {

        // Function to reorder characters using an index pattern
        fn reorder(data: &str) -> String {
            let pattern = [6, 7, 4, 5, 2, 3, 0, 1];
            pattern.iter().map(|&i| data.chars().nth(i).unwrap()).collect()
        }

        // Format and reorder each data string
        // Ensure data0, data1, data2, data3 are padded to 8 characters
        let final_data = [
            reorder(&format!("{:0>8}", raw_data0)),
            reorder(&format!("{:0>8}", raw_data1)),
            reorder(&format!("{:0>8}", raw_data2)),
            reorder(&format!("{:0>8}", raw_data3)),
        ]
        .concat();

        debug!("Job gauge update: {}", final_data);

        match job_name {
            "DNC" => rotation_dancer::update_job_gauge(job_details, &final_data),
            _ => {},
        }
    }
}