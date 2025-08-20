use windows_sys::Win32::{
    System::Memory::{VirtualAlloc, VirtualFree, MEM_COMMIT, PAGE_READWRITE, MEM_RELEASE, VirtualProtect, PAGE_READONLY},
};

// Windows堆内存分配器实现
pub struct Win32HeapAllocator;

unsafe impl core::alloc::GlobalAlloc for Win32HeapAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        VirtualAlloc(
            core::ptr::null_mut(),
            layout.size(),
            MEM_COMMIT,
            PAGE_READWRITE,
        ) as _
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        // Windows通常不单独释放已提交的页面
        VirtualFree(ptr as *mut _, 0, MEM_RELEASE);
    }
}




