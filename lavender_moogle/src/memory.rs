extern crate winapi;

use std::ptr::null_mut;
use std::sync::{Arc, Mutex};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::NULL;
use winapi::um::memoryapi::{CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_ALL_ACCESS};
use winapi::um::winnt::{HANDLE, PAGE_READWRITE};

use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use log::{debug, error};

use crate::types::{SpellDetails, JobDetails};


static NEXT_SPELL_STR: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::from("none")));

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
    let cname: Vec<u16> = OsStr::new(MEMORY_NAME).encode_wide().chain(Some(0)).collect();
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
        Err(format!("Failed to create file mapping: {}", std::io::Error::last_os_error()))
    } else {
        Ok(handle)
    }
}

fn map_view_of_file(handle: HANDLE) -> Result<*mut u8, String> {
    let ptr = unsafe { MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, MEMORY_SIZE) };

    if ptr.is_null() {
        Err(format!("Failed to map view of file: {}", std::io::Error::last_os_error()))
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
// WARNING: puts the same spell in all the shared memory slots
pub fn populate_shared_memory(spell: SpellDetails) -> Result<(), String> {
    let _lock = MMAP_MUTEX.lock().unwrap();

    // Retrieve the global handle
    let shared_handle = SHARED_MEMORY_HANDLE.lock().unwrap();

    match *shared_handle {
        Some(handle) => {
            let ptr = map_view_of_file(handle.0)?;

            unsafe {
                for i in 0..4 {
                    ptr.add(i * 2).write(spell.modifier as u8);
                    ptr.add(i * 2 + 1).write(spell.key as u8);
                }
            }

            unsafe { UnmapViewOfFile(ptr as *const _) };

            Ok(())
        }
        None => Err("Shared memory handle is not initialized".to_string()),
    }
}

pub fn communicate_next_spellcast(job_details: &mut JobDetails, spell_name: &str) {
    let mut saved_next_spell_name = NEXT_SPELL_STR.lock().unwrap();
    if *saved_next_spell_name != spell_name {
        debug!("Updating next spellcast to {}", spell_name);
        *saved_next_spell_name = spell_name.to_string(); // copy
        if let Some(next_spell_details) = job_details.get_spell(spell_name).cloned() {
            let _ = populate_shared_memory(next_spell_details);
        } else {
            error!("Spell {} not found for shared memory", spell_name);
        }
    }
}
