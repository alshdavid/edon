use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_unref_threadsafe_function".as_bytes();
type SIGNATURE = fn(env: napi_env, func: napi_threadsafe_function) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_unref_threadsafe_function(
  env: napi_env,
  func: napi_threadsafe_function,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, func)
}
