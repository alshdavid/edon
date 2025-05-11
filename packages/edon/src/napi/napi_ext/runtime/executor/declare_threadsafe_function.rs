#![allow(dead_code)]
use std::ffi::c_void;
use std::ptr;

use libnode_sys as napi_sys;

pub fn declare_threadsafe_function(
  env: napi_sys::napi_env,
  name: &str,
  callback: unsafe extern "C" fn(
    env: napi_sys::napi_env,
    js_callback: napi_sys::napi_value,
    context: *mut c_void,
    data: *mut c_void,
  ),
) -> *mut napi_sys::napi_threadsafe_function__ {
  unsafe {
    let raw_tsfn = {
      let mut async_resource_name = ptr::null_mut();
      napi_sys::napi_create_string_utf8(
        env,
        name.as_ptr().cast(),
        name.len() as isize,
        &mut async_resource_name,
      );

      let mut raw_tsfn: *mut napi_sys::napi_threadsafe_function__ = ptr::null_mut();
      napi_sys::napi_create_threadsafe_function(
        env,
        ptr::null_mut(),
        ptr::null_mut(),
        async_resource_name,
        0,
        1,
        ptr::null_mut(),
        None,
        ptr::null_mut(),
        Some(callback),
        &mut raw_tsfn,
      );

      raw_tsfn
    };

    napi_sys::napi_unref_threadsafe_function(env, raw_tsfn);
    // raw_tsfn as usize
    raw_tsfn
  }
}
