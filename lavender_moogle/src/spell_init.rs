// spell_init.rs
// manage the function relatives to spell list configuration
use calamine::{open_workbook_auto, DataType, Reader};
use chrono::Utc;
use std::collections::HashMap;
use std::error::Error;
use crate::types::{SpellDescription, SpellBook, JobSpells};

pub fn init_spell_book_from_excel(file_path: &str) -> Result<SpellBook, Box<dyn Error>> {
    let mut workbook = open_workbook_auto(file_path)?;
    let mut spell_book: SpellBook = HashMap::new();
    
    for sheet_name in workbook.sheet_names().to_owned() {
        if let Some(Ok(range)) = workbook.worksheet_range(&sheet_name) {
            let headers: Vec<String> = range.rows().next().unwrap_or(&[]).iter()
                .map(|h| h.to_string())
                .collect();
            

            let mut job_spells: JobSpells = HashMap::new();

            for row in range.rows().skip(1) {
                let mut spell_description = SpellDescription {
                    name: String::new(),
                    modifier: '\0',
                    key: '\0',
                    spell_type: String::new(),
                    duration: 0,
                    cooldown: 0,
                    give_status: vec![],
                    remove_status: vec![],
                    active_until: Utc::now(),
                    unavaible_before: Utc::now(),
                };

                for (col_idx, cell) in row.iter().enumerate() {
                    match headers.get(col_idx).map(|h| h.as_str()) {
                        Some("name") => spell_description.name = cell.to_string(),
                
                        Some("modifier") => {
                            spell_description.modifier = cell.to_string().chars().next().unwrap_or('\0');  // Default to null character if empty
                        }
                
                        Some("key") => {
                            spell_description.key = cell.to_string().chars().next().unwrap_or('\0');  // Default to null character if empty
                        }
                
                        Some("spell_type") => spell_description.spell_type = cell.to_string(),
                
                        Some("duration") => {
                            spell_description.duration = match cell {
                                DataType::Int(value) => *value as i32,
                                DataType::Float(value) => *value as i32,
                                DataType::Empty => 0,  // Default to 0 if empty
                                _ => cell.to_string().parse().unwrap_or(0),  // Fallback to parsing, default to 0 on failure
                            };
                        }
                
                        Some("cooldown") => {
                            spell_description.cooldown = match cell {
                                DataType::Int(value) => *value as i32,
                                DataType::Float(value) => *value as i32,
                                DataType::Empty => 0,  // Default to 0 if empty
                                _ => cell.to_string().parse().unwrap_or(0),  // Fallback to parsing, default to 0 on failure
                            };
                        }
                
                        Some("give_status") => {
                            if cell.is_empty() {
                                spell_description.give_status = Vec::new();  // Set to an empty vector if the cell is empty
                            } else {
                                spell_description.give_status = cell.to_string().split(';')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())  // Filter out any empty strings after splitting
                                    .collect();
                            }
                        }                        
                
                        Some("remove_status") => {
                            if cell.is_empty() {
                                spell_description.remove_status = Vec::new();  // Set to an empty vector if the cell is empty
                            } else {
                                spell_description.remove_status = cell.to_string().split(';')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())  // Optional: filter out any empty strings after splitting
                                    .collect();
                            }
                        }

                        _ => {}
                    }
                }

                // Insert the spell into the JobSpells struct HashMap
                job_spells.insert(spell_description.name.clone(), spell_description);
            }

            // Insert the JobSpells into the SpellBook
            spell_book.insert(sheet_name.clone(), job_spells);
        }
    }

    Ok(spell_book)
}

pub fn pretty_print_spell_book(spell_book: &SpellBook) {
    for (job_name, job_spells) in spell_book {
        println!("Job: {}", job_name);
        println!("{}", "=".repeat(20));

        for (spell_name, spell_description) in job_spells {
            println!("Spell Name: {}", spell_name);
            println!("  Modifier: {}", spell_description.modifier);
            println!("  Key: {}", spell_description.key);
            println!("  Type: {}", spell_description.spell_type);
            println!("  Duration: {} milliseconds", spell_description.duration);
            println!("  Cooldown: {} milliseconds", spell_description.cooldown);

            if !spell_description.give_status.is_empty() {
                println!("  Gives Buffs: {}", spell_description.give_status.join(", "));
            } else {
                println!("  Gives Buffs: None");
            }

            if !spell_description.remove_status.is_empty() {
                println!("  Removes Buffs: {}", spell_description.remove_status.join(", "));
            } else {
                println!("  Removes Buffs: None");
            }

            println!("  Active Until: {}", spell_description.active_until);
            println!("  Unavailable Before: {}", spell_description.unavaible_before);

            println!("{}", "-".repeat(20));
        }

        println!();
    }
}