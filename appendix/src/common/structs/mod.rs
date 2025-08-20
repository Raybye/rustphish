// #[cfg(not(feature = "noheap"))]
pub mod unsafety;

#[cfg(feature = "noheap")]
pub mod noheap;

pub trait Utf16{
    fn new(s: &str) -> Self;
    fn as_ptr(&self) -> *const u16;
    fn len(&self) -> usize;

    // 拼接多个 UTF-16 字符串片段
    fn join<'a>(parts: impl IntoIterator<Item = &'a str>) -> Self;
    fn join_str_utf16_str(s1: &str, u_s2: &[u16], s3: &str) -> Self;
}
