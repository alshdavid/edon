use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_remove_async_cleanup_hook".as_bytes();
type SIGNATURE = fn(remove_handle: napi_async_cleanup_hook_handle) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_remove_async_cleanup_hook(
  remove_handle: napi_async_cleanup_hook_handle
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(remove_handle)
}
