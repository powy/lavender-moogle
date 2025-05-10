extern crate winapi;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};

use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::NULL;
use winapi::um::memoryapi::{
    CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_ALL_ACCESS,
};
use winapi::um::winnt::{HANDLE, PAGE_READWRITE};

use lazy_static::lazy_static;
use log::{debug, error};
use once_cell::sync::Lazy;

use crate::types::{JobDetails, SpellDetails, SpellcastType};

static NEXT_SPELL_STR_MONO_STANDARD: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new(String::from("none")));
static NEXT_SPELL_STR_MONO_ALTERNATE: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new(String::from("none")));
static NEXT_SPELL_STR_AOE_STANDARD: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new(String::from("none")));
static NEXT_SPELL_STR_AOE_ALTERNATE: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new(String::from("none")));

const MEMORY_NAME: &str = "Global\\lavender_keys";
const MEMORY_SIZE: usize = 8;

// Define a newtype to wrap the HANDLE
#[derive(Debug, Clone, Copy)]
struct SafeHandle(HANDLE);

unsafe impl Send for SafeHandle {}
unsafe impl Sync for SafeHandle {}

lazy_static! {
    static ref MMAP_MUTEX: Mutex<()> = Mutex::new(());
    static ref SHARED_MEMORY_HANDLE: Arc<Mutex<Option<SafeHandle>>> = Arc::new(Mutex::new(None));
}

fn create_file_mapping() -> Result<HANDLE, String> {
    let cname: Vec<u16> = OsStr::new(MEMORY_NAME)
        .encode_wide()
        .chain(Some(0))
        .collect();
    let handle = unsafe {
        CreateFileMappingW(
            NULL,
            null_mut(),
            PAGE_READWRITE,
            0,
            MEMORY_SIZE as DWORD,
            cname.as_ptr(),
        )
    };

    if handle.is_null() {
        Err(format!(
            "Failed to create file mapping: {}",
            std::io::Error::last_os_error()
        ))
    } else {
        Ok(handle)
    }
}

fn map_view_of_file(handle: HANDLE) -> Result<*mut u8, String> {
    let ptr = unsafe { MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, MEMORY_SIZE) };

    if ptr.is_null() {
        Err(format!(
            "Failed to map view of file: {}",
            std::io::Error::last_os_error()
        ))
    } else {
        Ok(ptr as *mut u8)
    }
}

pub fn initialize_or_reset_shared_memory() -> Result<(), String> {
    let _lock = MMAP_MUTEX.lock().unwrap();

    let handle = create_file_mapping()?;
    let ptr = map_view_of_file(handle)?;

    unsafe {
        for i in 0..MEMORY_SIZE {
            ptr.add(i).write(0);
        }
    }

    unsafe { UnmapViewOfFile(ptr as *const _) };

    // Store the handle in the global variable
    let mut shared_handle = SHARED_MEMORY_HANDLE.lock().unwrap();
    *shared_handle = Some(SafeHandle(handle));

    Ok(())
}

// transfer the keybinding of a spell to the shared memory
// Expected positions:
// 0 = n (standard)
// 1 = a (maybe "no burst" or alternative)
// 2 = s (Maybe "AOE")
// 3 = c (maybe "AOE" + "no burst" or alternative)
pub fn populate_shared_memory(
    spell: SpellDetails,
    spellcast_type: SpellcastType,
) -> Result<(), String> {
    // debug!("Spellcast Type: {:?}", spellcast_type);

    let position = spellcast_type.to_position();

    if position > 3 {
        eprintln!("Invalid position {}: must be between 0 and 3", position);
        return Ok(()); // Do nothing, but not an error in the Result
    }

    let _lock = MMAP_MUTEX.lock().unwrap();
    let shared_handle = SHARED_MEMORY_HANDLE.lock().unwrap();

    match *shared_handle {
        Some(handle) => {
            let ptr = map_view_of_file(handle.0)?;

            unsafe {
                let offset = position * 2;
                ptr.add(offset).write(spell.modifier as u8);
                ptr.add(offset + 1).write(spell.key as u8);
            }

            unsafe { UnmapViewOfFile(ptr as *const _) };
            Ok(())
        }
        None => Err("Shared memory handle is not initialized".to_string()),
    }
}

pub fn communicate_next_spellcast(
    job_details: &mut JobDetails,
    spell_name: &str,
    aoe: bool,
    alternate: bool,
) {
    let spellcast_type = SpellcastType::from_flags(alternate, aoe);
    // debug!("Spellcast Type: {:?}", spellcast_type);

    let next_spell_str = match spellcast_type {
        SpellcastType::MonoStandard => &NEXT_SPELL_STR_MONO_STANDARD,
        SpellcastType::MonoAlternate => &NEXT_SPELL_STR_MONO_ALTERNATE,
        SpellcastType::AoeStandard => &NEXT_SPELL_STR_AOE_STANDARD,
        SpellcastType::AoeAlternate => &NEXT_SPELL_STR_AOE_ALTERNATE,
    };

    let mut saved_next_spell_name = next_spell_str.lock().unwrap();
    if *saved_next_spell_name != spell_name {
        debug!(
            "Updating next spellcast to {} with spellcast type {:?}",
            spell_name, spellcast_type
        );
        *saved_next_spell_name = spell_name.to_string(); // copy
        if let Some(next_spell_details) = job_details.get_spell(spell_name).cloned() {
            let _ = populate_shared_memory(next_spell_details, spellcast_type);
        } else {
            error!("Spell {} not found for shared memory", spell_name);
        }
    }
}
