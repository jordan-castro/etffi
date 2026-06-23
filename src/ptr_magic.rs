use core::ffi::c_void;
use std::panic::Location;

/// Box up a pointer into a raw pointer.
#[macro_export]
macro_rules! box_raw {
    ($item:expr) => {
        Box::into_raw(Box::new($item))
    };
}

/// A shared trait for converting from/to a pointer. Specifically a (* mut Self)
pub trait PtrMagic: Sized {
    /// Moves the object to the heap and returns a raw pointer.
    /// Caller owns this memory but don't worry about freeing it. The library frees it somewhere.
    fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    /// Get a direct *mut c_void
    fn into_void(self) -> *mut c_void {
        self.into_raw() as *mut c_void
    }

    #[track_caller]
    /// Safety: Only call this on a pointer created via `into_raw`.
    fn from_raw(ptr: *mut Self) -> Self {
        let location = Location::caller();
        assert!(!ptr.is_null(), "Attempted to own a null pointer. Stack: {}:{}:{}", location.file(), location.line(), location.column());
        unsafe { *Box::from_raw(ptr) }
    }

    #[track_caller]
    /// Build from a Ptr but only get a reference, this means that the caller will still own the memory
    unsafe fn from_borrow<'a>(ptr: *mut Self) -> &'a mut Self {
        let location = Location::caller();
        assert!(!ptr.is_null(), "Attempted to borrow a null pointer. Stack: {}:{}:{}", location.file(), location.line(), location.column());
        unsafe {
            &mut *ptr
        }
    }

    /// Completely unsafe and should only be used when cerrtain that type can be cast to Self
    unsafe fn from_borrow_void<'a>(ptr: *mut c_void) -> &'a mut Self {
        unsafe { Self::from_borrow(ptr as *mut Self) }
    }
}

/// Generic from_raw for ThreadLanguageState
fn generic_from_raw<T>(pointer: *mut T) {
    let _: T = unsafe { *Box::from_raw(pointer) };
}

/// Wraps a pointer and allows it to be passed around threads.
/// 
/// It can optionally free the pointer.
pub struct ThreadSafePointer<T> {
    pointer: *mut T,
    free_on_drop: bool
}

impl<T> ThreadSafePointer<T> {
    pub fn new(pointer: *mut T) -> Self {
        ThreadSafePointer { pointer, free_on_drop: false }
    }

    pub fn new_owned(pointer: *mut T) -> Self {
        ThreadSafePointer { pointer, free_on_drop: true }
    }

    pub fn get_ptr(&self) -> *mut T {
        self.pointer
    }
}

impl<T> Drop for ThreadSafePointer<T> {
    fn drop(&mut self) {
        if !self.free_on_drop {
            return;
        }
        if self.pointer.is_null() {
            return;
        }
        generic_from_raw::<T>(self.pointer);
        self.pointer = core::ptr::null_mut();
    }
}

unsafe impl<T> Sync for ThreadSafePointer<T> {}
unsafe impl<T> Send for ThreadSafePointer<T> {}