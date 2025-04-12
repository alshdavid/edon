use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "uv_run".as_bytes();
type SIGNATURE = fn(loop_: *mut uv_loop_s, mode: uv_run_mode) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn uv_run(
  loop_: *mut uv_loop_s,
  mode: uv_run_mode,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(loop_, mode)
}
