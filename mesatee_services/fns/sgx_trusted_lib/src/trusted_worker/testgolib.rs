#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;
extern crate libc;

use std::ffi::{CStr, CString};
use std::vec;

#[derive(Debug)]
#[repr(C)]
struct GoString {
    a: *const libc::c_char,
    b: i64,
}

#[derive(Debug)]
#[repr(C)]
struct GoSlice {
    data: *mut libc::c_void,
    len: libc::c_longlong,
    cap: libc::c_longlong,
}

#[repr(C)]
struct AddMultiRet_return {
    r0: *const libc::c_char,
    r1: libc::c_longlong,
}

extern "C" {
    fn Add(a: libc::c_longlong, b: libc::c_longlong) -> libc::c_longlong;
    fn AddArray(a: GoSlice, b: GoSlice, c: *mut GoSlice);
    fn AddString(a: GoString, b: GoString) -> *mut libc::c_char;
    fn AddMultiRet() -> AddMultiRet_return;
}

pub fn run() {
    let result = unsafe { Add(10i64, 12i64) };
    debug!("{:?}", result);

    let mut a_arr = vec![20i64, 2i64].into_boxed_slice();
    let a_slice = GoSlice {
        data: a_arr.as_mut_ptr() as *mut libc::c_void,
        len: a_arr.len() as libc::c_longlong,
        cap: a_arr.len() as libc::c_longlong,
    };

    let mut b_arr = vec![138877474747i64, 2i64].into_boxed_slice();
    let b_slice = GoSlice {
        data: b_arr.as_mut_ptr() as *mut libc::c_void,
        len: b_arr.len() as libc::c_longlong,
        cap: b_arr.len() as libc::c_longlong,
    };
    debug!("{:?}", a_slice);
    debug!("{:?}", b_slice);

    let mut c_arr = vec![0i64, 0i64].into_boxed_slice();
    let mut c_slice = Box::new(GoSlice {
        data: c_arr.as_mut_ptr() as *mut libc::c_void,
        len: c_arr.len() as libc::c_longlong,
        cap: c_arr.len() as libc::c_longlong,
    });
    // 在后面还要再次取回，因此要保存直到手动释放
    std::mem::forget(c_arr);

    debug!("begin to call");
    unsafe { AddArray(a_slice, b_slice, &mut *c_slice) };
    debug!("end call");

    let c_arr = unsafe {
        std::vec::Vec::from_raw_parts(
            c_slice.data as *mut i64,
            c_slice.len as usize,
            c_slice.cap as usize,
        )
    };
    debug!("{:?}", c_arr);
    std::mem::drop(c_arr);

    let c_path = CString::new("hello duanbing cstring").expect("CString::new failed");
    let ptr = c_path.as_ptr();
    let go_string = GoString {
        a: ptr,
        b: c_path.as_bytes().len() as i64,
    };
    
    let c_path = CString::new("hello duanbing cstring").expect("CString::new failed");
    let ptr = c_path.as_ptr();
    let go_string2 = GoString {
        a: ptr,
        b: c_path.as_bytes().len() as i64,
    };

    debug!("go go ");
    let res = unsafe { AddString(go_string, go_string2) };
    // https://stackoverflow.com/questions/24145823/how-do-i-convert-a-c-string-into-a-rust-string-and-back-via-ffi
    // 需要释放内存 TODO
    let c_str = unsafe { CStr::from_ptr(res) };
    debug!("res: {:?}", c_str.to_str().unwrap());

    let res = unsafe { AddMultiRet() };
    let c_str = unsafe { CStr::from_ptr(res.r0) };
    debug!("res: {:?}, len={}", c_str.to_str().unwrap(), res.r1);
}
