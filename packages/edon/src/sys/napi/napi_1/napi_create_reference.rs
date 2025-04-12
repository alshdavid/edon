use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_create_reference".as_bytes();
type SIGNATURE =
  fn(env: napi_env, value: napi_value, initial_refcount: u32, result: *mut napi_ref) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_create_reference(
  env: napi_env,
  value: napi_value,
  initial_refcount: u32,
  result: *mut napi_ref,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env,
    value,
    initial_refcount,
    result,
  )
}
