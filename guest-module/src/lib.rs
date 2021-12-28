use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

use base64_helper::{encode_with_termination, decode_with_termination};

#[no_mangle]
pub extern fn allocate(len: usize) -> *mut c_void {
    // create a new mutable buffer with capacity `len`
    let mut buf = Vec::with_capacity(len);
    // take a mutable pointer to the buffer
    let ptr = buf.as_mut_ptr();
    // dont destroy mem block after func
    std::mem::forget(buf);
    // return the pointer
    return ptr;
}

#[no_mangle]
pub extern fn deallocate(pointer: *mut c_void, capacity: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(pointer, 0, capacity);
    }
}

#[no_mangle]
pub extern fn greet(subject: *mut c_char) -> *mut c_char {
    let subject = unsafe { CStr::from_ptr(subject).to_bytes().to_vec() };
    let mut output = b"Hello, ".to_vec();
    output.extend(&subject);
    output.extend(&[b'!']);

    unsafe { CString::from_vec_unchecked(output) }.into_raw()
}

#[no_mangle]
pub extern fn reverse_array(subject: *mut c_char) -> *mut c_char {
    let subject = unsafe { CStr::from_ptr(subject).to_bytes().to_vec() };
    let mut decoded = decode_with_termination(&subject).unwrap();
    decoded.reverse();

    let output:Vec<u8> = encode_with_termination(decoded).unwrap().into_bytes();

    unsafe { CString::from_vec_unchecked(output).into_raw() }
}

/* Currently not compilable (on this machine atleast), because Multi-Value compilation from rustc is highly experimental and linker fails for me
    https://github.com/rust-lang/rust/issues/73755
cargo rustc -Zmultitarget --target=wasm32-wasi -- -C target-feature=+multivalue

// Needs to be compiled with rustc nightly and command found in Makefile to actually omit the multi value
#[no_mangle]
pub extern fn greet(subject: *mut c_char) -> (i32, i32) {
    let subject = unsafe { CStr::from_ptr(subject).to_bytes().to_vec() };
    let mut output = b"Hello, ".to_vec();
    output.extend(&subject);
    output.extend(&[b'!']);

    let len = output.len();

    (unsafe { CString::from_vec_unchecked(output) }.into_raw() as i32, len as i32)
} */