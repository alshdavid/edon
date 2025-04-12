use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_uint;
use std::ffi::c_void;
use std::sync::OnceLock;

const SYMBOL: &[u8] = "node_module_register".as_bytes();
type SIGNATURE = fn(mod_: *mut node_module);

#[allow(non_camel_case_types)]
pub type node_addon_register_func = Option<
  unsafe extern "C" fn(
    // v8::Local<v8::Object> exports,
    // v8::Local<v8::Value> module,
    // void* priv
  ),
>;

#[allow(non_camel_case_types)]
pub type node_addon_context_register_func = Option<
  unsafe extern "C" fn(
    // v8::Local<v8::Object> exports,
    // v8::Local<v8::Value> module,
    // v8::Local<v8::Context> context,
    // void* priv
  ),
>;

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct node_module {
  pub nm_version: c_int,
  pub nm_flags: c_uint,
  pub nm_dso_handle: *mut c_void,
  pub nm_filename: *const c_char,
  pub nm_register_func: node_addon_register_func,
  pub nm_context_register_func: node_addon_context_register_func,
  pub nm_modname: *const c_char,
  pub nm_priv: *mut c_void,
  pub nm_link: *mut node_module,
}

static CACHE: OnceLock<super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn node_module_register(module: *mut node_module) {
  CACHE.get_or_init(|| super::super::libnode::libnode_sym(SYMBOL))(module)
}
