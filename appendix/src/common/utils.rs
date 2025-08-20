/// Computes the DJB2 hash for the given buffer
pub fn dbj2_hash(buffer: &[u8]) -> u32 {
    let mut hsh: u32 = 5381;
    let mut iter: usize = 0;
    let mut cur: u8;

    while iter < buffer.len() {
        cur = buffer[iter];

        if cur == 0 {
            iter += 1;
            continue;
        }

        if cur >= ('a' as u8) {
            cur -= 0x20;
        }

        hsh = ((hsh << 5).wrapping_add(hsh)) + cur as u32;
        iter += 1;
    }
    hsh
}

/// Calculates the length of a C-style null-terminated string.
pub fn get_cstr_len(pointer: *const char) -> usize {
    let mut tmp: u64 = pointer as u64;

    unsafe {
        while *(tmp as *const u8) != 0 {
            tmp += 1;
        }
    }

    (tmp - pointer as u64) as _
}

pub fn string_length_w(string: *const u16) -> usize {
    unsafe {
        let mut string2 = string;
        while !(*string2).is_null() {
            string2 = string2.add(1);
        }
        string2.offset_from(string) as usize
    }
}

// Utility function for checking null terminator for u8 and u16
trait IsNull {
    fn is_null(&self) -> bool;
}

impl IsNull for u16 {
    fn is_null(&self) -> bool {
        *self == 0
    }
}


pub fn strip_utf16_null(s: &[u16]) -> &[u16] {
    if let Some(pos) = s.iter().rposition(|&c| c != 0) {
        &s[..=pos]
    } else {
        &[]
    }
}

pub fn str_to_utf16(s: &str) -> alloc::vec::Vec<u16> {
    let mut buf: alloc::vec::Vec<u16> = s.encode_utf16().collect();
    buf.push(0);
    buf
}

/// Helper function to convert a hex digit (0-15) into its corresponding ASCII character.
///
/// # Returns
/// The corresponding ASCII character as a `u16`.
fn to_hex_char(digit: u16) -> u16 {
    match digit {
        0..=9 => '0' as u16 + digit,
        10..=15 => 'a' as u16 + (digit - 10),
        _ => 0,
    }
}
