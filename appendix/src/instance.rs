//通过魔术值（magic number）在进程堆（heap）中查找并获取一个全局共享结构体，封装NTAPI函数表
use crate::debug_println;

#[cfg(feature = "debug")]
use crate::debug::Writer;

use crate::{
    // k32::Kernel32, 
    native::ntapi::NtDll, 
    native::ntdef::find_peb
};

// A magic number to identify a valid `Instance` struct
pub const INSTANCE_MAGIC: u32 = 0x19191919;

#[repr(C)]
// The main structure holding system API modules and the magic value
pub struct Instance {
    pub magic: u32,       // Unique value to identify a valid instance
    // pub k32: Kernel32,    // Kernel32 API functions
    pub ntdll: NtDll,     // NtDll API functions
    // pub winsock: Winsock, // Winsock API functions
}

impl Instance {
    pub fn new() -> Self {
        Instance {
            magic: INSTANCE_MAGIC,
            // k32: Kernel32::new(),
            ntdll: NtDll::new(),
            // winsock: Winsock::new(),
        }
    }
}

/// Attempts to locate the global `Instance` by scanning process heaps and
/// returns a mutable reference to it if found.
pub unsafe fn get_instance() -> Option<&'static mut Instance> {
    let peb = find_peb(); // Locate the PEB (Process Environment Block)
    let process_heaps = (*peb).process_heaps;
    let number_of_heaps = (*peb).number_of_heaps as usize;

    for i in 0..number_of_heaps {
        let heap = *process_heaps.add(i);
        if !heap.is_null() {
            let instance = &mut *(heap as *mut Instance);
            if instance.magic == INSTANCE_MAGIC {
                return Some(instance); // Return the instance if the magic value matches
            }
        }
    }
    None
}
