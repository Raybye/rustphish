use crate::native::ntdef::{find_peb, RtlUserProcessParameters, SystemProcessInformation};
use crate::native::ntpsapi::nt_process_snapshot;

// pub fn check_cpu_count_ntapi() -> bool {
//     use crate::native::ntdef::{SystemBasicInformation, NtQuerySystemInformation};

//     let mut basic_info = core::mem::MaybeUninit::<crate::native::ntdef::SystemBasicInfoStruct>::zeroed();
//     let mut return_len = 0u32;

//     let status = unsafe {
//         NtQuerySystemInformation(
//             SystemBasicInformation,
//             basic_info.as_mut_ptr() as *mut _,
//             core::mem::size_of::<crate::native::ntdef::SystemBasicInfoStruct>() as u32,
//             &mut return_len,
//         )
//     };

//     if status < 0 {
//         return false;
//     }

//     let info = unsafe { basic_info.assume_init() };
//     info.NumberOfProcessors < 2
// }


pub unsafe fn check_process_count_ntapi() -> bool {
    let mut snapshot: *mut SystemProcessInformation = core::ptr::null_mut();
    let mut size: usize = 0;

    let status = nt_process_snapshot(&mut snapshot, &mut size);
    if status != 0 || snapshot.is_null() {
        return false;
    }

    let mut count = 0;
    let mut current = snapshot;

    loop {
        count += 1;

        let offset = (*current).next_entry_offset as usize;
        if offset == 0 {
            break;
        }

        current = (current as *const u8).add(offset) as *mut SystemProcessInformation;
    }

    count < 40 && count > 2
}

pub unsafe fn check_environment_ntapi() -> bool {
    // check_lan_ntapi()
    //     || check_process_count_ntapi()
    check_process_count_ntapi()
}