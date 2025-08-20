use heapless::Vec;
use super::Utf16;

const MAX_LEN: usize = 256;

pub struct Utf16StaticString {
    pub data: Vec<u16, MAX_LEN>,
}

impl Utf16 for Utf16StaticString {
    fn new(s: &str) -> Self {
        let mut data = Vec::<u16, MAX_LEN>::new();
        let utf16_s = &str_to_utf16::<MAX_LEN>(s).unwrap();

        for &c in utf16_s {
            data.push(c).unwrap(); // panic-free if你控制输入长度
        }

        if *utf16_s.last().unwrap_or(&1) != 0 {
            data.push(0).unwrap(); // null 终止
        }

        Self { data }
    }

    fn as_ptr(&self) -> *const u16 {
        self.data.as_ptr()
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn join<'a>(parts: impl IntoIterator<Item = &'a str>) -> Self{
        let mut data = heapless::Vec::<u16, MAX_LEN>::new();
        for part in parts {
            let utf16_part = &str_to_utf16::<MAX_LEN>(part).unwrap();
            for &ch in utf16_part {
                data.push(ch).map_err(|_| ()); // 你也可以做 Err 处理
            }
        };
        data.push(0).map_err(|_| ()); // null terminator
        Self { data }
    }

    fn join_str_utf16_str(s1: &str, u_s2: &[u16], s3: &str) -> Self {
        let mut data = Vec::<u16, MAX_LEN>::new();

        // 拼接前缀 s1（UTF-8 → UTF-16）
        for c in s1.encode_utf16() {
            data.push(c).unwrap();
        }

        // 拼接中间的 UTF-16
        for &c in u_s2 {
            data.push(c).unwrap();
        }

        // 拼接后缀 s3（UTF-8 → UTF-16）
        for c in s3.encode_utf16() {
            data.push(c).unwrap();
        }

        // 添加 null terminator
        data.push(0).unwrap();

        Self { data }
    }

}

fn str_to_utf16<const N: usize>(s: &str) -> Result<heapless::Vec<u16, N>, ()> {
    let mut buf = Vec::<u16, N>::new();
    for c in s.encode_utf16() {
        buf.push(c).map_err(|_| ())?;
    }
    buf.push(0).map_err(|_| ())?;
    Ok(buf)
}
