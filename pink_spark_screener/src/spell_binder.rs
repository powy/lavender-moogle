// spell_binder.rs
// manage the function relatives to spell list configuration
// and execution of association
use calamine::{open_workbook, Reader, Xlsx};
use std::collections::HashMap;

use crate::types::SpellBinding;

// Note: there is some special cases where we consider the top line going up to 12
// This is to follow the logic of mmo mouse and typical hotbar size
// Convert the whole bar to AAZERTY
fn convert_to_char(value: &str) -> char {
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
        _ => if trimmed_value.len() == 1 {
            trimmed_value.chars().next().unwrap_or('i')
        } else {
            'i'
        },
    }
}

pub fn read_spells_from_excel(file_path: &str) -> HashMap<String, SpellBinding> {
    let mut workbook: Xlsx<_> = open_workbook(file_path).expect("Cannot open Excel file");

    let sheet = workbook.worksheet_range("spells").expect("Cannot find sheet").expect("Cannot read sheet");

    let mut spell_dict = HashMap::new();
    let mut col_indices = HashMap::new();

    if let Some(header_row) = sheet.rows().next() {
        for (i, cell) in header_row.iter().enumerate() {
            match cell.get_string() {
                Some("output") => col_indices.insert("output", i),
                Some("modifier") => col_indices.insert("modifier", i),
                Some("key") => col_indices.insert("key", i),
                _ => None,
            };
        }
    }

    for row in sheet.rows().skip(1) { // Skip the header row
        if let (Some(output_idx), Some(modifier_idx), Some(key_idx)) = (
            col_indices.get("output"),
            col_indices.get("modifier"),
            col_indices.get("key"),
        ) {
            if let (Some(output), Some(modifier_str), Some(key_str)) = (
                row.get(*output_idx),
                row.get(*modifier_idx),
                row.get(*key_idx),
            ) {
                let output = output.to_string();
                let modifier_char = convert_to_char(&modifier_str.to_string());
                let key_char = convert_to_char(&key_str.to_string());
                let spell = SpellBinding { modifier: modifier_char, key: key_char };
                spell_dict.insert(output.clone(), spell);
                // println!("Inserted spell: output = {}, modifier = {}, key = {}", output, modifier_char, key_char);
            }
        }
    }

    spell_dict
}

pub fn get_spell_bindings_from_text(text: &str, spell_dict: &HashMap<String, SpellBinding>) -> [SpellBinding; 4] {
    // Split the text into lines and remove the first line (which is the spec)
    let lines: Vec<&str> = text.lines().collect();
    // The default binding structure is all "invalid" (i)
    let mut bindings = [SpellBinding { modifier: 'i', key: 'i' }; 4];
    
    // Not enough lines. Returning initial bindings
    if lines.len() < 5 {
        return bindings
    }

    for i in 1..5 {
        // The line comes from a scan, it should therefore be trimmed in case.
        let line = lines[i].trim();
        
        if let Some(binding) = spell_dict.get(line) {
            // Bindings has one less element than the lines, the indexes different
            bindings[i - 1] = binding.clone();
        }
    }

    bindings
}
