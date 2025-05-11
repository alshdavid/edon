use std::convert::From;
use std::ffi::c_void;
use std::ffi::CString;
use std::ptr;

use bitflags::bitflags;
use libnode_sys;

use crate::napi::bindgen_runtime::FromNapiValue;
use crate::napi::bindgen_runtime::This;
use crate::napi::bindgen_runtime::ToNapiValue;
use crate::napi::Callback;
use crate::napi::Env;
use crate::napi::NapiRaw;
use crate::napi::Result;

#[derive(Copy, Clone)]
pub struct PropertyClosures {
  pub setter_closure: *mut c_void,
  pub getter_closure: *mut c_void,
}

impl Default for PropertyClosures {
  fn default() -> Self {
    Self {
      setter_closure: ptr::null_mut(),
      getter_closure: ptr::null_mut(),
    }
  }
}

#[derive(Clone)]
pub struct Property {
  pub name: CString,
  getter: libnode_sys::napi_callback,
  setter: libnode_sys::napi_callback,
  method: libnode_sys::napi_callback,
  attrs: PropertyAttributes,
  value: libnode_sys::napi_value,
  pub(crate) is_ctor: bool,
  pub(crate) closures: PropertyClosures,
}

impl Default for Property {
  fn default() -> Self {
    Property {
      name: Default::default(),
      getter: Default::default(),
      setter: Default::default(),
      method: Default::default(),
      attrs: Default::default(),
      value: ptr::null_mut(),
      is_ctor: Default::default(),
      closures: PropertyClosures::default(),
    }
  }
}

bitflags! {
  #[derive(Debug, Copy, Clone)]
  pub struct PropertyAttributes: i32 {
    const Default = libnode_sys::PropertyAttributes::default;
    const Writable = libnode_sys::PropertyAttributes::writable;
    const Enumerable = libnode_sys::PropertyAttributes::enumerable;
    const Configurable = libnode_sys::PropertyAttributes::configurable;
    const Static = libnode_sys::PropertyAttributes::static_;
  }
}

impl Default for PropertyAttributes {
  fn default() -> Self {
    PropertyAttributes::Configurable | PropertyAttributes::Enumerable | PropertyAttributes::Writable
  }
}

impl From<PropertyAttributes> for libnode_sys::napi_property_attributes {
  fn from(value: PropertyAttributes) -> Self {
    value.bits()
  }
}

impl Property {
  pub fn new(name: &str) -> Result<Self> {
    Ok(Property {
      name: CString::new(name)?,
      ..Default::default()
    })
  }

  pub fn with_name(
    mut self,
    name: &str,
  ) -> Self {
    self.name = CString::new(name).unwrap();
    self
  }

  pub fn with_method(
    mut self,
    callback: Callback,
  ) -> Self {
    self.method = Some(callback);
    self
  }

  pub fn with_getter(
    mut self,
    callback: Callback,
  ) -> Self {
    self.getter = Some(callback);
    self
  }

  pub fn with_getter_closure<R, F>(
    mut self,
    callback: F,
  ) -> Self
  where
    F: 'static + Fn(Env, This) -> Result<R>,
    R: ToNapiValue,
  {
    let boxed_callback = Box::new(callback);
    let closure_data_ptr: *mut F = Box::into_raw(boxed_callback);
    self.closures.getter_closure = closure_data_ptr.cast();

    let fun = crate::napi::trampoline_getter::<R, F>;
    self.getter = Some(fun);
    self
  }

  pub fn with_setter(
    mut self,
    callback: Callback,
  ) -> Self {
    self.setter = Some(callback);
    self
  }

  pub fn with_setter_closure<F, V>(
    mut self,
    callback: F,
  ) -> Self
  where
    F: 'static + Fn(crate::napi::Env, This, V) -> Result<()>,
    V: FromNapiValue,
  {
    let boxed_callback = Box::new(callback);
    let closure_data_ptr: *mut F = Box::into_raw(boxed_callback);
    self.closures.setter_closure = closure_data_ptr.cast();

    let fun = crate::napi::trampoline_setter::<V, F>;
    self.setter = Some(fun);
    self
  }

  pub fn with_property_attributes(
    mut self,
    attributes: PropertyAttributes,
  ) -> Self {
    self.attrs = attributes;
    self
  }

  pub fn with_value<T: NapiRaw>(
    mut self,
    value: &T,
  ) -> Self {
    self.value = unsafe { T::raw(value) };
    self
  }

  pub(crate) fn raw(&self) -> libnode_sys::napi_property_descriptor {
    let closures = Box::into_raw(Box::new(self.closures));
    libnode_sys::napi_property_descriptor {
      utf8name: self.name.as_ptr(),
      name: ptr::null_mut(),
      method: self.method,
      getter: self.getter,
      setter: self.setter,
      value: self.value,
      attributes: self.attrs.into(),
      data: closures.cast(),
    }
  }

  pub fn with_ctor(
    mut self,
    callback: Callback,
  ) -> Self {
    self.method = Some(callback);
    self.is_ctor = true;
    self
  }
}
