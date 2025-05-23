use std::any::TypeId;
use std::ops::Deref;
use std::ops::DerefMut;

use libnode_sys;

use super::FromNapiValue;
use super::ToNapiValue;
use super::TypeName;
use super::ValidateNapiValue;
use crate::napi::check_status;
use crate::napi::Error;
use crate::napi::Status;
use crate::napi::TaggedObject;

pub struct External<T: 'static> {
  obj: *mut TaggedObject<T>,
  size_hint: usize,
  pub adjusted_size: i64,
}

unsafe impl<T: 'static + Send> Send for External<T> {}
unsafe impl<T: 'static + Sync> Sync for External<T> {}

impl<T: 'static> TypeName for External<T> {
  fn type_name() -> &'static str {
    "External"
  }

  fn value_type() -> crate::napi::ValueType {
    crate::napi::ValueType::External
  }
}

impl<T: 'static> From<T> for External<T> {
  fn from(t: T) -> Self {
    External::new(t)
  }
}

impl<T: 'static> ValidateNapiValue for External<T> {}

impl<T: 'static> External<T> {
  pub fn new(value: T) -> Self {
    Self {
      obj: Box::into_raw(Box::new(TaggedObject::new(value))),
      size_hint: 0,
      adjusted_size: 0,
    }
  }

  /// Turn a raw pointer (from napi) pointing to the inner `*mut TaggedObject<T>` into a reference inner object.
  ///
  /// # Safety
  /// The `unknown_tagged_object` raw pointer must point to an `TaggedObject<T>` struct, which
  /// is essentially the pointer which napi-rs returns to the NAPI api.
  unsafe fn from_raw_impl(
    unknown_tagged_object: *mut std::ffi::c_void
  ) -> Option<*mut TaggedObject<T>> {
    let type_id = unknown_tagged_object as *const TypeId;
    if unsafe { *type_id } == TypeId::of::<T>() {
      let tagged_object = unknown_tagged_object as *mut TaggedObject<T>;
      Some(tagged_object)
    } else {
      None
    }
  }

  /// Turn a raw pointer (from napi) pointing to the inner `*mut TaggedObject<T>` into a reference inner object.
  ///
  /// # Safety
  /// The `unknown_tagged_object` raw pointer must point to an `TaggedObject<T>` struct, which
  /// is essentially the pointer which napi-rs returns to the NAPI api.
  ///
  /// Additionally, you must ensure that there are no other live mutable references to the `T` for
  /// the duration of the lifetime of the returned mutable reference.
  pub unsafe fn inner_from_raw_mut(
    unknown_tagged_object: *mut std::ffi::c_void
  ) -> Option<&'static mut T> {
    Self::from_raw_impl(unknown_tagged_object)
      .and_then(|tagged_object| unsafe { (*tagged_object).object.as_mut() })
  }

  /// Turn a raw pointer (from napi) pointing to the inner `*mut TaggedObject<T>` into a reference inner object.
  ///
  /// # Safety
  /// The `unknown_tagged_object` raw pointer must point to an `TaggedObject<T>` struct, which
  /// is essentially the pointer which napi-rs returns to the NAPI api.
  ///
  /// Additionally, you must ensure that there are no other live mutable references to the `T` for
  /// the duration of the lifetime of the returned immutable reference.
  pub unsafe fn inner_from_raw(unknown_tagged_object: *mut std::ffi::c_void) -> Option<&'static T> {
    Self::from_raw_impl(unknown_tagged_object)
      .and_then(|tagged_object| unsafe { (*tagged_object).object.as_ref() })
  }

  /// `size_hint` is a value to tell Node.js GC how much memory is used by this `External` object.
  ///
  /// If getting the exact `size_hint` is difficult, you can provide an approximate value, it's only effect to the GC.
  ///
  /// If your `External` object is not effect to GC, you can use `External::new` instead.
  pub fn new_with_size_hint(
    value: T,
    size_hint: usize,
  ) -> Self {
    Self {
      obj: Box::into_raw(Box::new(TaggedObject::new(value))),
      size_hint,
      adjusted_size: 0,
    }
  }
}

impl<T: 'static> FromNapiValue for External<T> {
  unsafe fn from_napi_value(
    env: libnode_sys::napi_env,
    napi_val: libnode_sys::napi_value,
  ) -> crate::napi::Result<Self> {
    let mut unknown_tagged_object = std::ptr::null_mut();
    check_status!(
      unsafe { libnode_sys::napi_get_value_external(env, napi_val, &mut unknown_tagged_object) },
      "Failed to get external value"
    )?;

    let type_id = unknown_tagged_object as *const TypeId;
    if unsafe { *type_id } == TypeId::of::<T>() {
      let tagged_object = unknown_tagged_object as *mut TaggedObject<T>;
      Ok(Self {
        obj: tagged_object,
        size_hint: 0,
        adjusted_size: 0,
      })
    } else {
      Err(Error::new(
        Status::InvalidArg,
        "T on `get_value_external` is not the type of wrapped object".to_owned(),
      ))
    }
  }
}

impl<T: 'static> AsRef<T> for External<T> {
  fn as_ref(&self) -> &T {
    unsafe { Box::leak(Box::from_raw(self.obj)).object.as_ref().unwrap() }
  }
}

impl<T: 'static> AsMut<T> for External<T> {
  fn as_mut(&mut self) -> &mut T {
    unsafe { Box::leak(Box::from_raw(self.obj)).object.as_mut().unwrap() }
  }
}

impl<T: 'static> Deref for External<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    self.as_ref()
  }
}

impl<T: 'static> DerefMut for External<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.as_mut()
  }
}

impl<T: 'static> ToNapiValue for External<T> {
  unsafe fn to_napi_value(
    env: libnode_sys::napi_env,
    mut val: Self,
  ) -> crate::napi::Result<libnode_sys::napi_value> {
    let mut napi_value = std::ptr::null_mut();
    check_status!(
      unsafe {
        libnode_sys::napi_create_external(
          env,
          val.obj as *mut _,
          Some(crate::napi::raw_finalize::<T>),
          Box::into_raw(Box::new(Some(val.size_hint as i64))) as *mut _,
          &mut napi_value,
        )
      },
      "Create external value failed"
    )?;
    #[cfg(not(target_family = "wasm"))]
    {
      let mut adjusted_external_memory_size = std::mem::MaybeUninit::new(0);

      if val.size_hint != 0 {
        check_status!(
          unsafe {
            libnode_sys::napi_adjust_external_memory(
              env,
              val.size_hint as i64,
              adjusted_external_memory_size.as_mut_ptr(),
            )
          },
          "Adjust external memory failed"
        )?;
      };

      val.adjusted_size = unsafe { adjusted_external_memory_size.assume_init() };
    }

    Ok(napi_value)
  }
}
