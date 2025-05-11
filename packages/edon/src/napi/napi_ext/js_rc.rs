#![allow(dead_code)]
use std::cell::OnceCell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr;

use libnode_sys as napi_sys;

use crate::napi::check_status;
use crate::napi::Env;
use crate::napi::NapiRaw;
use crate::napi::NapiValue;

thread_local! {
  static NAPI_ENV: OnceCell<*mut napi_sys::napi_env__> = OnceCell::default();
}

// unsafe impl<T> Send for JsRc<T> {}

pub struct JsRc<T> {
  raw_ref: napi_sys::napi_ref,
  _inner: PhantomData<T>,
}

impl<T: NapiValue> JsRc<T> {
  pub fn new(
    env: &Env,
    inner: T,
  ) -> crate::napi::Result<Self> {
    Self::new_raw(env.raw(), unsafe { inner.raw() })
  }

  fn new_raw(
    raw_env: napi_sys::napi_env,
    inner_raw: napi_sys::napi_value,
  ) -> crate::napi::Result<Self> {
    let raw_env = NAPI_ENV.with(|f| *f.get_or_init(|| raw_env));

    let obj = {
      let mut raw_value = ptr::null_mut();
      check_status!(unsafe {
        napi_sys::napi_create_array_with_length(raw_env, 1, &mut raw_value)
      })?;
      raw_value
    };

    check_status!(unsafe { napi_sys::napi_set_element(raw_env, obj, 0, inner_raw.cast()) })?;

    let mut raw_ref = ptr::null_mut();
    check_status!(unsafe { napi_sys::napi_create_reference(raw_env, obj, 1, &mut raw_ref) })?;

    Ok(Self {
      raw_ref,
      _inner: Default::default(),
    })
  }

  pub fn get(&self) -> crate::Result<T> {
    let env_raw = NAPI_ENV.with(|f| *f.get().unwrap());

    let mut napi_value = ptr::null_mut();
    unsafe { napi_sys::napi_get_reference_value(env_raw, self.raw_ref, &mut napi_value) };

    let mut raw_value = ptr::null_mut();
    unsafe { napi_sys::napi_get_element(env_raw, napi_value, 0, &mut raw_value) };

    let value = unsafe { T::from_raw_unchecked(env_raw, raw_value) };

    Ok(value)
  }
}

impl<T> Clone for JsRc<T> {
  fn clone(&self) -> Self {
    let env_raw = NAPI_ENV.with(|f| *f.get().unwrap());

    unsafe { napi_sys::napi_reference_ref(env_raw, self.raw_ref.cast(), ptr::null_mut()) };

    Self {
      raw_ref: self.raw_ref.clone(),
      _inner: self._inner.clone(),
    }
  }
}

impl<T> Drop for JsRc<T> {
  fn drop(&mut self) {
    let env_raw = NAPI_ENV.with(|f| *f.get().unwrap());
    let mut count = 0;
    unsafe { napi_sys::napi_reference_unref(env_raw, self.raw_ref.cast(), &mut count) };
    if count == 0 {
      unsafe { napi_sys::napi_delete_reference(env_raw, self.raw_ref.cast()) };
    }
  }
}

impl<T: NapiValue> Deref for JsRc<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    let value = self.get().unwrap();
    let value = Box::leak(Box::new(value));
    value
  }
}

impl<T: NapiValue> NapiRaw for JsRc<T> {
  unsafe fn raw(&self) -> napi_sys::napi_value {
    self.get().unwrap().raw()
  }
}

impl<T: NapiValue> NapiValue for JsRc<T> {
  unsafe fn from_raw(
    env: napi_sys::napi_env,
    value: napi_sys::napi_value,
  ) -> crate::napi::Result<Self> {
    JsRc::new_raw(env, value)
  }

  unsafe fn from_raw_unchecked(
    env: napi_sys::napi_env,
    value: napi_sys::napi_value,
  ) -> Self {
    JsRc::new_raw(env, value).unwrap()
  }
}

pub trait JsRcExt<T: NapiRaw> {
  /// Wraps the JavaScript value in a reference counted container
  /// that prevents Nodejs's GC from dropping it
  fn into_rc(
    self,
    env: &Env,
  ) -> crate::napi::Result<JsRc<T>>;
}

impl<T: NapiValue> JsRcExt<T> for T {
  fn into_rc(
    self,
    env: &Env,
  ) -> crate::napi::Result<JsRc<T>> {
    JsRc::new(env, self)
  }
}
