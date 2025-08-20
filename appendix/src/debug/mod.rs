// #[cfg(feature = "ntdll")]
// pub mod k32;

// use alloc::string::String;
// use alloc::string::ToString;

#[cfg(feature = "debug")]
use core::char::decode_utf16;

use core::fmt::{self, Write};

#[cfg(not(feature = "debug"))]
pub struct Writer {
    pub handle: *mut core::ffi::c_void,
}

#[cfg(feature = "debug")]
pub struct Writer;


impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            let mut bytes_written: u32 = 0;
            unsafe extern "system" {
                fn WriteFile(
                    hFile: *mut core::ffi::c_void,
                    lpBuffer: *const core::ffi::c_void,
                    nNumberOfBytesToWrite: u32,
                    lpNumberOfBytesWritten: *mut u32,
                    lpOverlapped: *mut core::ffi::c_void,
                ) -> i32;
            }

            WriteFile(
                (-11i32 as usize) as *mut core::ffi::c_void, // STD_OUTPUT_HANDLE
                s.as_ptr() as *const core::ffi::c_void,
                s.len() as u32,
                &mut bytes_written,
                core::ptr::null_mut(),
            );
        }
        Ok(())
    }
}


#[cfg(feature = "debug")]
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = writeln!($crate::Writer, $($arg)*);
    }};
}

#[cfg(feature = "debug")]
pub fn utf16_to_utf8_str(buf: &[u16]) -> heapless::String<256> {
    let mut s = heapless::String::new();
    for c in decode_utf16(buf.iter().copied()) {
        match c {
            Ok(ch) => s.push(ch).ok(),
            Err(_) => s.push('\u{FFFD}').ok(),
        };
    }
    s
}

// #[macro_export]
// macro_rules! debug_println {
//     // Case 1: Just printing a message without any values
//     ($msg:expr) => {{
//         let mut bytes_written: u32 = 0;
//         unsafe {
//             ($crate::instance::get_instance().unwrap().k32.write_file)(
//                 -11i32 as u32 as *mut core::ffi::c_void, // Handle for STD_OUTPUT_HANDLE
//                 $msg.as_ptr() as *const core::ffi::c_void, // Pointer to the message
//                 $msg.len() as u32,                       // Length of the message
//                 &mut bytes_written,                      // Bytes written output
//                 core::ptr::null_mut(),
//             );
//         }
//     }};

//     // Case 2: Printing an NTSTATUS value (i32) in hex, always
//     ($msg:expr, $val:expr) => {{
//         // Format the NTSTATUS value as hex
//         let formatted_val = $crate::debug::itoa_hex_i32($val); // Convert i32 directly to hex

//         // Concatenate the message and the formatted value
//         let full_msg = [$msg, &formatted_val, "\n"].concat();
//         let mut bytes_written: u32 = 0;

//         unsafe {
//             ($crate::instance::get_instance().unwrap().k32.write_file)(
//                 -11i32 as u32 as *mut core::ffi::c_void, // Handle for STD_OUTPUT_HANDLE
//                 full_msg.as_ptr() as *const core::ffi::c_void, // Pointer to the full message
//                 full_msg.len() as u32,                   // Length of the full message
//                 &mut bytes_written,                      // Bytes written output
//                 core::ptr::null_mut(),
//             );
//         }
//     }};

//     // Case 3: Printing a usize value in decimal or hex
//     ($msg:expr, $val:expr, $as_hex:expr) => {{
//         // Format the value as decimal or hexadecimal based on $as_hex
//         let formatted_val = if $as_hex {
//             $crate::debug::itoa_hex($val as usize) // Convert usize to hex
//         } else {
//             $crate::debug::itoa($val as usize) // Convert usize to decimal
//         };

//         // Concatenate the message and the formatted value
//         let full_msg = [$msg, &formatted_val, "\n"].concat();
//         let mut bytes_written: u32 = 0;

//         unsafe {
//             ($crate::instance::get_instance().unwrap().k32.write_file)(
//                 -11i32 as u32 as *mut core::ffi::c_void, // Handle for STD_OUTPUT_HANDLE
//                 full_msg.as_ptr() as *const core::ffi::c_void, // Pointer to the full message
//                 full_msg.len() as u32,                   // Length of the full message
//                 &mut bytes_written,                      // Bytes written output
//                 core::ptr::null_mut(),
//             );
//         }
//     }};
// }

// /// Converts a 32-bit integer (i32) to a hexadecimal string in uppercase.
// pub fn itoa_hex_i32(value: i32) -> String {
//     let hex_digits = b"0123456789ABCDEF"; // Array of uppercase hexadecimal digits.
//     let mut result = String::new();
//     let mut num = value as u32; // Treat the value as u32 while preserving the bit pattern of i32.

//     if num == 0 {
//         return "0x0".to_string(); // Special case for the value 0.
//     }

//     // Convert the number to hexadecimal by repeatedly dividing by 16 and collecting the remainder.
//     while num > 0 {
//         let digit = (num % 16) as usize;
//         result.push(hex_digits[digit] as char);
//         num /= 16;
//     }

//     // Pad with zeros to ensure the hexadecimal number is represented by 8 digits.
//     // NTSTATUS codes are always represented with 8 characters in hexadecimal.
//     while result.len() < 8 {
//         result.push('0');
//     }

//     // Append the "0x" prefix at the beginning.
//     result.push_str("x0");

//     // Reverse the string to correct the order (since we built it from the least significant digit).
//     result.chars().rev().collect()
// }

// // Helper function to convert a memory address (pointer) to a hexadecimal string with 0x prefix and uppercase
// pub fn itoa_hex(value: usize) -> String {
//     let hex_digits = b"0123456789ABCDEF"; // Uppercase hexadecimal digits
//     let mut result = String::new();
//     let mut num = value;

//     if num == 0 {
//         return "0x0".to_string(); // Special case for 0
//     }

//     // Convert number to hexadecimal
//     while num > 0 {
//         let digit = (num % 16) as usize;
//         result.push(hex_digits[digit] as char);
//         num /= 16;
//     }

//     // Add the '0x' prefix
//     result.push_str("x0");

//     // Reverse the string since it's built backwards and return the final result
//     result.chars().rev().collect()
// }
