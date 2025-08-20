use alloc::alloc::{alloc, dealloc, Layout};
use super::Utf16;
use alloc::vec::Vec;
use core::ptr::write;
use core::ptr::copy_nonoverlapping;

/// Heap-allocated UTF-16 string with automatic deallocation
pub struct Utf16String {
    pub ptr: *mut u16,
    pub len: usize,
}

// 堆手工管理
impl Utf16 for Utf16String {
    fn new(s: &str) -> Self {
        let utf16_vec = &str_to_utf16(s);
                
        let len = s.len() + 1;
        let layout = Layout::array::<u16>(len).unwrap();
        let ptr = unsafe { alloc(layout) as *mut u16 };

        let mut i = 0;
        for &c in utf16_vec {
            // 不安全解引用 越界写可能导致内存破坏
            unsafe { *ptr.add(i) = c };
            i += 1;
        }

        unsafe { *ptr.add(i) = 0 }; // 添加null终止

        Self { ptr, len }
    }

    fn as_ptr(&self) -> *const u16 {
        self.ptr
    }

    fn len(&self) -> usize {
        self.len
    }

    /// 拼接多个 UTF-16 片段，构造一个新的 Utf16String
    fn join<'a>(parts: impl IntoIterator<Item = &'a str>) -> Self{

        // 收集为动态数组
        let utf16_parts: Vec<Vec<u16>> = parts.into_iter()
            .map(str_to_utf16)
            .collect();
        let total_len: usize = utf16_parts
            .iter()
            .map(|s| s.len() - 1)  // 去掉每段的 null terminator
            .sum::<usize>() + 1;   // 加上整体的 null terminator

        // 分配堆内存
        let layout = Layout::array::<u16>(total_len).unwrap();
        let ptr = unsafe { alloc(layout) as *mut u16 };
        assert!(!ptr.is_null());

        let mut offset = 0;
        for part in &utf16_parts {
            // 拷贝每段（去除结尾 null）
            for &c in &part[..part.len() - 1] {
                unsafe { write(ptr.add(offset), c); }
                offset += 1;
            }
        }

        // null 终止
        unsafe { write(ptr.add(offset), 0); }

        Self { ptr, len: total_len }
    }

    fn join_str_utf16_str(s1: &str, u_s2: &[u16], s3: &str) -> Self {
        
        let prefix = Self::new(s1);
        let suffix = Self::new(s3);

        let path_len = u_s2.len();
        let total_len = prefix.len() + path_len + suffix.len(); // 包含 prefix/suffix 的 null

        let layout = Layout::array::<u16>(total_len).unwrap();
        let cmd_ptr = unsafe { alloc(layout) as *mut u16 };

        unsafe {
            let mut offset = 0;

            copy_nonoverlapping(prefix.as_ptr(), cmd_ptr.add(offset), prefix.len() - 1);
            offset += prefix.len() - 1;

            copy_nonoverlapping(u_s2.as_ptr(), cmd_ptr.add(offset), path_len);
            offset += path_len;

            copy_nonoverlapping(suffix.as_ptr(), cmd_ptr.add(offset), suffix.len());
        }

        Self { ptr: cmd_ptr, len: total_len }
    }

}

// 自动释放
impl Drop for Utf16String {
    fn drop(&mut self) {
        let layout = Layout::array::<u16>(self.len).unwrap();
        unsafe { dealloc(self.ptr as *mut u8, layout) };
    }
}

unsafe impl Send for Utf16String {}
unsafe impl Sync for Utf16String {}

fn str_to_utf16(s: &str) -> alloc::vec::Vec<u16> {
    let mut buf: alloc::vec::Vec<u16> = s.encode_utf16().collect();
    buf.push(0);
    buf
}
