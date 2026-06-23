/// Instead of using `Bytes` use this for C interop.
#[repr(C)]
pub struct CBytes {
    data: *mut u8,
    size: usize
}