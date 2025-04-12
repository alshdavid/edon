use std::ffi::c_char;
use std::ffi::c_void;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_define_class".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  utf8name: *const c_char,
  length: isize,
  constructor: napi_callback,
  data: *mut c_void,
  property_count: usize,
  properties: *const napi_property_descriptor,
  result: *mut napi_value,
) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_define_class(
  env: napi_env,
  utf8name: *const c_char,
  length: isize,
  constructor: napi_callback,
  data: *mut c_void,
  property_count: usize,
  properties: *const napi_property_descriptor,
  result: *mut napi_value,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env,
    utf8name,
    length,
    constructor,
    data,
    property_count,
    properties,
    result,
  )
}
