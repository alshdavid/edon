use std::convert::TryFrom;
use std::ffi::c_void;
use std::ptr;

use super::check_status;
use super::Value;
use libnode_sys;
use crate::napi::Env;
use crate::napi::Error;
use crate::napi::Result;

pub struct JsObject(pub(crate) Value);
impl From<Value> for JsObject {
  fn from(value: Value) -> Self {
    Self(value)
  }
}

pub struct FinalizeContext<T: 'static, Hint: 'static> {
  pub env: Env,
  pub value: T,
  pub hint: Hint,
}

impl JsObject {
  pub fn add_finalizer<T, Hint, F>(
    &mut self,
    native: T,
    finalize_hint: Hint,
    finalize_cb: F,
  ) -> Result<()>
  where
    T: 'static,
    Hint: 'static,
    F: FnOnce(FinalizeContext<T, Hint>) + 'static,
  {
    let mut maybe_ref = ptr::null_mut();
    let wrap_context = Box::leak(Box::new((native, finalize_cb, ptr::null_mut())));
    check_status!(unsafe {
      libnode_sys::napi_add_finalizer(
        self.0.env,
        self.0.value,
        wrap_context as *mut _ as *mut c_void,
        Some(
          finalize_callback::<T, Hint, F>
            as unsafe extern "C" fn(
              env: libnode_sys::napi_env,
              finalize_data: *mut c_void,
              finalize_hint: *mut c_void,
            ),
        ),
        Box::leak(Box::new(finalize_hint)) as *mut _ as *mut c_void,
        &mut maybe_ref, // Note: this does not point to the boxed one…
      )
    })?;
    wrap_context.2 = maybe_ref;
    Ok(())
  }
}

unsafe extern "C" fn finalize_callback<T, Hint, F>(
  raw_env: libnode_sys::napi_env,
  finalize_data: *mut c_void,
  finalize_hint: *mut c_void,
) where
  T: 'static,
  Hint: 'static,
  F: FnOnce(FinalizeContext<T, Hint>),
{
  let (value, callback, raw_ref) =
    unsafe { *Box::from_raw(finalize_data as *mut (T, F, libnode_sys::napi_ref)) };
  let hint = unsafe { *Box::from_raw(finalize_hint as *mut Hint) };
  let env = unsafe { Env::from_raw(raw_env) };
  callback(FinalizeContext { env, value, hint });
  if !raw_ref.is_null() {
    let status = unsafe { libnode_sys::napi_delete_reference(raw_env, raw_ref) };
    debug_assert!(
      status == libnode_sys::Status::napi_ok,
      "Delete reference in finalize callback failed"
    );
  }
}

pub enum KeyCollectionMode {
  IncludePrototypes,
  OwnOnly,
}

impl TryFrom<libnode_sys::napi_key_collection_mode> for KeyCollectionMode {
  type Error = Error;

  fn try_from(value: libnode_sys::napi_key_collection_mode) -> Result<Self> {
    match value {
      libnode_sys::KeyCollectionMode::include_prototypes => Ok(Self::IncludePrototypes),
      libnode_sys::KeyCollectionMode::own_only => Ok(Self::OwnOnly),
      _ => Err(Error::new(
        crate::napi::Status::InvalidArg,
        format!("Invalid key collection mode: {value}"),
      )),
    }
  }
}

impl From<KeyCollectionMode> for libnode_sys::napi_key_collection_mode {
  fn from(value: KeyCollectionMode) -> Self {
    match value {
      KeyCollectionMode::IncludePrototypes => libnode_sys::KeyCollectionMode::include_prototypes,
      KeyCollectionMode::OwnOnly => libnode_sys::KeyCollectionMode::own_only,
    }
  }
}

pub enum KeyFilter {
  AllProperties,
  Writable,
  Enumerable,
  Configurable,
  SkipStrings,
  SkipSymbols,
}

impl TryFrom<libnode_sys::napi_key_filter> for KeyFilter {
  type Error = Error;

  fn try_from(value: libnode_sys::napi_key_filter) -> Result<Self> {
    match value {
      libnode_sys::KeyFilter::all_properties => Ok(Self::AllProperties),
      libnode_sys::KeyFilter::writable => Ok(Self::Writable),
      libnode_sys::KeyFilter::enumerable => Ok(Self::Enumerable),
      libnode_sys::KeyFilter::configurable => Ok(Self::Configurable),
      libnode_sys::KeyFilter::skip_strings => Ok(Self::SkipStrings),
      libnode_sys::KeyFilter::skip_symbols => Ok(Self::SkipSymbols),
      _ => Err(Error::new(
        crate::napi::Status::InvalidArg,
        format!("Invalid key filter [{value}]"),
      )),
    }
  }
}

impl From<KeyFilter> for libnode_sys::napi_key_filter {
  fn from(value: KeyFilter) -> Self {
    match value {
      KeyFilter::AllProperties => libnode_sys::KeyFilter::all_properties,
      KeyFilter::Writable => libnode_sys::KeyFilter::writable,
      KeyFilter::Enumerable => libnode_sys::KeyFilter::enumerable,
      KeyFilter::Configurable => libnode_sys::KeyFilter::configurable,
      KeyFilter::SkipStrings => libnode_sys::KeyFilter::skip_strings,
      KeyFilter::SkipSymbols => libnode_sys::KeyFilter::skip_symbols,
    }
  }
}

pub enum KeyConversion {
  KeepNumbers,
  NumbersToStrings,
}

impl TryFrom<libnode_sys::napi_key_conversion> for KeyConversion {
  type Error = Error;

  fn try_from(value: libnode_sys::napi_key_conversion) -> Result<Self> {
    match value {
      libnode_sys::KeyConversion::keep_numbers => Ok(Self::KeepNumbers),
      libnode_sys::KeyConversion::numbers_to_strings => Ok(Self::NumbersToStrings),
      _ => Err(Error::new(
        crate::napi::Status::InvalidArg,
        format!("Invalid key conversion [{value}]"),
      )),
    }
  }
}

impl From<KeyConversion> for libnode_sys::napi_key_conversion {
  fn from(value: KeyConversion) -> Self {
    match value {
      KeyConversion::KeepNumbers => libnode_sys::KeyConversion::keep_numbers,
      KeyConversion::NumbersToStrings => libnode_sys::KeyConversion::numbers_to_strings,
    }
  }
}
