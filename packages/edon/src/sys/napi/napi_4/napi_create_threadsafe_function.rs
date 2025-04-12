use std::ffi::c_void;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_create_threadsafe_function".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  func: napi_value,
  async_resource: napi_value,
  async_resource_name: napi_value,
  max_queue_size: usize,
  initial_thread_count: usize,
  thread_finalize_data: *mut c_void,
  thread_finalize_cb: napi_finalize,
  context: *mut c_void,
  call_js_cb: napi_threadsafe_function_call_js,
  result: *mut napi_threadsafe_function,
) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_create_threadsafe_function(
  env: napi_env,
  func: napi_value,
  async_resource: napi_value,
  async_resource_name: napi_value,
  max_queue_size: usize,
  initial_thread_count: usize,
  thread_finalize_data: *mut c_void,
  thread_finalize_cb: napi_finalize,
  context: *mut c_void,
  call_js_cb: napi_threadsafe_function_call_js,
  result: *mut napi_threadsafe_function,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env,
    func,
    async_resource,
    async_resource_name,
    max_queue_size,
    initial_thread_count,
    thread_finalize_data,
    thread_finalize_cb,
    context,
    call_js_cb,
    result,
  )
}
