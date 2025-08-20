use core::ffi::c_void;
use core::mem::MaybeUninit;
use windows_sys::Win32::Foundation::NTSTATUS;
use crate::winapi::windef::OSVERSIONINFOEXW;

#[repr(C)]
pub struct OSVERSIONINFOEXW {
    pub dwOSVersionInfoSize: u32,
    pub dwMajorVersion: u32,
    pub dwMinorVersion: u32,
    pub dwBuildNumber: u32,
    pub dwPlatformId: u32,
    pub szCSDVersion: [u16; 128],
    pub wServicePackMajor: u16,
    pub wServicePackMinor: u16,
    pub wSuiteMask: u16,
    pub wProductType: u8,
    pub wReserved: u8,
}

unsafe extern "system" {
    fn RtlGetVersion(lpVersionInformation: *mut OSVERSIONINFOEXW) -> NTSTATUS;
}

pub unsafe fn get_windows_version() -> Option<(u32, u32, u32)> {
    let mut info = MaybeUninit::<OSVERSIONINFOEXW>::zeroed();
    // (*info.as_mut_ptr()).dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOEXW>() as u32;
    let info_mut = info.assume_init_mut();
    info_mut.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOEXW>() as u32;

    // let status = RtlGetVersion(info.as_mut_ptr());
    let status = RtlGetVersion(info_mut as *mut _);

    if status == 0 {
        let info = info.assume_init();
        Some((info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber))
    } else {
        None
    }
}