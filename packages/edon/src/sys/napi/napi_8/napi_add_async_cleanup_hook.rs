use std::ffi::c_void;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_add_async_cleanup_hook".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  hook: napi_async_cleanup_hook,
  arg: *mut c_void,
  remove_handle: *mut napi_async_cleanup_hook_handle,
) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_add_async_cleanup_hook(
  env: napi_env,
  hook: napi_async_cleanup_hook,
  arg: *mut c_void,
  remove_handle: *mut napi_async_cleanup_hook_handle,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env,
    hook,
    arg,
    remove_handle,
  )
}
