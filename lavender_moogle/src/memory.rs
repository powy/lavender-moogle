extern crate winapi;

use std::ptr::null_mut;
use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::NULL;
use winapi::um::memoryapi::{CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_ALL_ACCESS};
use winapi::um::winnt::{HANDLE, PAGE_READWRITE};
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use crate::types::SpellDescription;

const MEMORY_NAME: &str = "Global\\lavender_keys";
const MEMORY_SIZE: usize = 8;

lazy_static! {
    static ref MMAP_MUTEX: Mutex<()> = Mutex::new(());
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

pub fn initialize_or_reset_shared_memory() -> Result<HANDLE, String> {
    let _lock = MMAP_MUTEX.lock().unwrap();

    let handle = create_file_mapping()?;
    let ptr = map_view_of_file(handle)?;

    unsafe {
        for i in 0..MEMORY_SIZE {
            ptr.add(i).write(0);
        }
    }

    unsafe { UnmapViewOfFile(ptr as *const _) };
    
    Ok(handle)
}

pub fn populate_shared_memory(handle: HANDLE, input: [SpellDescription; 4]) -> Result<(), String> {
    let _lock = MMAP_MUTEX.lock().unwrap();

    let ptr = map_view_of_file(handle)?;

    unsafe {
        for (i, spell) in input.iter().enumerate() {
            ptr.add(i * 2).write(spell.modifier as u8);
            ptr.add(i * 2 + 1).write(spell.key as u8);
        }
    }

    unsafe { UnmapViewOfFile(ptr as *const _) };

    Ok(())
}
