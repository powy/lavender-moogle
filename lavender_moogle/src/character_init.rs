// character_init.rs
// manage the function relatives to spell list configuration
use crate::types::{CharacterDetails, JobDetails, SpellDetails};
use calamine::{open_workbook_auto, DataType, Reader};
use chrono::Utc;
use log::{debug, warn};
use std::collections::HashMap;
use std::error::Error;

pub fn init_character_details_from_excel(
    file_path: &str,
) -> Result<CharacterDetails, Box<dyn Error>> {
    let mut workbook = open_workbook_auto(file_path)?;
    let mut character_details: CharacterDetails = HashMap::new();

    for sheet_name in workbook.sheet_names().to_owned() {
        if let Some(Ok(range)) = workbook.worksheet_range(&sheet_name) {
            let headers: Vec<String> = range
                .rows()
                .next()
                .unwrap_or(&[])
                .iter()
                .map(|h| h.to_string())
                .collect();

            let mut job_details = JobDetails {
                spells: HashMap::new(),
                mana: 0,
                secondary_bar: 0,
                tertiary_bar: 0,
                stacks_a: 0,
                stacks_b: 0,
                weaves: 0,
                opener: true,
            };

            for row in range.rows().skip(1) {
                let mut spell_details = SpellDetails {
                    name: String::new(),
                    modifier: '\0',
                    key: '\0',
                    spell_type: String::new(),
                    duration: 0,
                    cooldown: 0,
                    give_status: vec![],
                    remove_status: vec![],
                    active_until: Utc::now(),
                    unavailable_before: Utc::now(),
                    mana: 0,
                    secondary_bar: 0,
                    tertiary_bar: 0,
                    stacks_a: 0,
                    stacks_b: 0,
                    max_charges: 0,
                };

                for (col_idx, cell) in row.iter().enumerate() {
                    match headers.get(col_idx).map(|h| h.as_str()) {
                        Some("name") => {
                            spell_details.name = cell.to_string();
                            debug!("spell : {}, job {}", spell_details.name, sheet_name);
                        }

                        Some("modifier") => {
                            spell_details.modifier = convert_to_char(cell.to_string());
                            // Default to null character if empty
                        }

                        Some("key") => {
                            spell_details.key = convert_to_char(cell.to_string());
                            // Default to null character if empty
                        }

                        Some("spell_type") => spell_details.spell_type = cell.to_string(),

                        Some("duration") => {
                            spell_details.duration = match cell {
                                DataType::Int(value) => *value as i32,
                                DataType::Float(value) => *value as i32,
                                DataType::Empty => 0, // Default to 0 if empty
                                _ => cell.to_string().parse().unwrap_or(0), // Fallback to parsing, default to 0 on failure
                            };
                        }

                        Some("cooldown") => {
                            spell_details.cooldown = match cell {
                                DataType::Int(value) => *value as i32,
                                DataType::Float(value) => *value as i32,
                                DataType::Empty => 0, // Default to 0 if empty
                                _ => cell.to_string().parse().unwrap_or(0), // Fallback to parsing, default to 0 on failure
                            };
                        }

                        Some("give_status") => {
                            if cell.is_empty() {
                                spell_details.give_status = Vec::new(); // Set to an empty vector if the cell is empty
                            } else {
                                spell_details.give_status = cell
                                    .to_string()
                                    .split(';')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty()) // Filter out any empty strings after splitting
                                    .collect();
                            }
                        }

                        Some("remove_status") => {
                            if cell.is_empty() {
                                spell_details.remove_status = Vec::new(); // Set to an empty vector if the cell is empty
                            } else {
                                spell_details.remove_status = cell
                                    .to_string()
                                    .split(';')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty()) // Optional: filter out any empty strings after splitting
                                    .collect();
                            }
                        }

                        Some("mana") => {
                            spell_details.mana = match cell {
                                DataType::Int(value) => *value as i32,
                                DataType::Float(value) => *value as i32,
                                DataType::Empty => 0, // Default to 0 if empty
                                _ => cell.to_string().parse().unwrap_or(0), // Fallback to parsing, default to 0 on failure
                            };
                        }

                        Some("secondary_bar") => {
                            spell_details.secondary_bar = match cell {
                                DataType::Int(value) => *value as i32,
                                DataType::Float(value) => *value as i32,
                                DataType::Empty => 0, // Default to 0 if empty
                                _ => cell.to_string().parse().unwrap_or(0), // Fallback to parsing, default to 0 on failure
                            };
                            if spell_details.secondary_bar != 0 {
                                debug!(
                                    "spell : {}, secondary_bar: {}",
                                    spell_details.name, spell_details.secondary_bar
                                );
                            }
                        }

                        Some("tertiary_bar") => {
                            spell_details.tertiary_bar = match cell {
                                DataType::Int(value) => *value as i32,
                                DataType::Float(value) => *value as i32,
                                DataType::Empty => 0, // Default to 0 if empty
                                _ => cell.to_string().parse().unwrap_or(0), // Fallback to parsing, default to 0 on failure
                            };
                        }

                        Some("stacks_a") => {
                            spell_details.stacks_a = match cell {
                                DataType::Int(value) => *value as i32,
                                DataType::Float(value) => *value as i32,
                                DataType::Empty => 0, // Default to 0 if empty
                                _ => cell.to_string().parse().unwrap_or(0), // Fallback to parsing, default to 0 on failure
                            };

                            if spell_details.stacks_a != 0 {
                                debug!(
                                    "spell : {}, stack a: {}",
                                    spell_details.name, spell_details.stacks_a
                                );
                            }
                        }

                        Some("stacks_b") => {
                            spell_details.stacks_b = match cell {
                                DataType::Int(value) => *value as i32,
                                DataType::Float(value) => *value as i32,
                                DataType::Empty => 0, // Default to 0 if empty
                                _ => cell.to_string().parse().unwrap_or(0), // Fallback to parsing, default to 0 on failure
                            };

                            if spell_details.stacks_b != 0 {
                                debug!(
                                    "spell : {}, stack b: {}",
                                    spell_details.name, spell_details.stacks_b
                                );
                            }
                        }

                        Some("max_charges") => {
                            // Parse the value from Excel cell, handling different data types
                            let mut parsed_value = match cell {
                                DataType::Int(value) => *value as i32,
                                DataType::Float(value) => *value as i32,
                                DataType::Empty => 0, // Default to 0 if empty
                                _ => cell.to_string().parse().unwrap_or(0), // Fallback to parsing, default to 0 on failure
                            };

                            // Validation: cap at 3, floor at 0
                            if parsed_value > 3 {
                                warn!(
                                    "Spell '{}': max_charges value {} exceeds maximum of 3. Capping at 3.",
                                    spell_details.name, parsed_value
                                );
                                parsed_value = 3;
                            } else if parsed_value < 0 {
                                warn!(
                                    "Spell '{}': max_charges value {} is negative. Setting to 0.",
                                    spell_details.name, parsed_value
                                );
                                parsed_value = 0;
                            }

                            spell_details.max_charges = parsed_value;

                            // Debug log for non-zero charges (similar to stacks_b pattern)
                            if spell_details.max_charges != 0 {
                                debug!(
                                    "spell: {}, max_charges: {}",
                                    spell_details.name, spell_details.max_charges
                                );
                            }
                        }

                        _ => {}
                    }
                }

                if !spell_details.name.trim().is_empty() {
                    job_details.insert_spell(spell_details.name.clone(), spell_details);
                }
            }

            // Insert the JobDetails into the CharacterDetails
            character_details.insert(sheet_name.clone(), job_details);
        }
    }

    Ok(character_details)
}

// Note: there is some special cases where we consider the top line going up to 12
// This is to follow the logic of mmo mouse and typical hotbar size
// Convert the whole bar to AAZERTY
fn convert_to_char(value: String) -> char {
    let trimmed_value = value.trim();
    match trimmed_value {
        "1" => '&',
        "2" => 'é',
        "3" => '\"',
        "4" => '\'',
        "5" => '(',
        "6" => '-',
        "7" => 'è',
        "8" => '_',
        "9" => 'ç',
        "10" => '0',
        "11" => ')',
        "12" => '=',
        _ => {
            if trimmed_value.len() == 1 {
                trimmed_value.chars().next().unwrap_or('i')
            } else {
                'i'
            }
        }
    }
}

// pub fn pretty_print_character_details(character_details: &CharacterDetails) {
//     for (job_name, job_details) in character_details {
//         println!("Job: {}", job_name);
//         println!("{}", "=".repeat(20));

//         for (spell_name, spell_details) in job_details {
//             println!("Spell Name: {}", spell_name);
//             println!("  Modifier: {}", spell_details.modifier);
//             println!("  Key: {}", spell_details.key);
//             println!("  Type: {}", spell_details.spell_type);
//             println!("  Duration: {} milliseconds", spell_details.duration);
//             println!("  Cooldown: {} milliseconds", spell_details.cooldown);

//             if !spell_details.give_status.is_empty() {
//                 println!("  Gives Buffs: {}", spell_details.give_status.join(", "));
//             } else {
//                 println!("  Gives Buffs: None");
//             }

//             if !spell_details.remove_status.is_empty() {
//                 println!("  Removes Buffs: {}", spell_details.remove_status.join(", "));
//             } else {
//                 println!("  Removes Buffs: None");
//             }

//             println!("  Active Until: {}", spell_details.active_until);
//             println!("  Unavailable Before: {}", spell_details.unavailable_before);

//             println!("{}", "-".repeat(20));
//         }

//         println!();
//     }
// }
