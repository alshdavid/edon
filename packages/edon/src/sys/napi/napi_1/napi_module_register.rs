use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_module_register".as_bytes();
type SIGNATURE = fn(mod_: *mut napi_module) -> napi_status;
static CACHE: OnceLock<super::super::super::library::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_module_register(mod_: *mut napi_module) -> napi_status {
  CACHE.get_or_init(|| super::super::super::library::get_sym(SYMBOL).unwrap())(mod_)
}
