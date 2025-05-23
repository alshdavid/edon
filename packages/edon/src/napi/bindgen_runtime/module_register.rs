use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::CStr;
use std::ptr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::RwLock;
use std::thread::ThreadId;

use libnode_sys;
use once_cell::sync::Lazy;

use crate::napi::check_status;
use crate::napi::check_status_or_throw;
use crate::napi::Env;
use crate::napi::JsError;
use crate::napi::JsFunction;
use crate::napi::Property;
use crate::napi::Result;
use crate::napi::Value;
use crate::napi::ValueType;

pub type ExportRegisterCallback =
  unsafe fn(libnode_sys::napi_env) -> Result<libnode_sys::napi_value>;
pub type ModuleExportsCallback =
  unsafe fn(env: libnode_sys::napi_env, exports: libnode_sys::napi_value) -> Result<()>;

#[repr(transparent)]
pub(crate) struct PersistedPerInstanceHashMap<K, V>(RwLock<HashMap<K, V>>);

impl<K, V> PersistedPerInstanceHashMap<K, V> {
  pub(crate) fn from_hashmap(hashmap: HashMap<K, V>) -> Self {
    Self(RwLock::new(hashmap))
  }

  #[allow(clippy::mut_from_ref)]
  pub(crate) fn borrow_mut<F, R>(
    &self,
    f: F,
  ) -> R
  where
    F: FnOnce(&mut HashMap<K, V>) -> R,
  {
    let mut write_lock = self.0.write().unwrap();
    f(&mut *write_lock)
  }
}

impl<K, V> Default for PersistedPerInstanceHashMap<K, V> {
  fn default() -> Self {
    Self(RwLock::new(HashMap::default()))
  }
}

type ModuleRegisterCallback =
  RwLock<Vec<(Option<&'static str>, (&'static str, ExportRegisterCallback))>>;

type ModuleClassProperty = PersistedPerInstanceHashMap<
  &'static str,
  HashMap<Option<&'static str>, (&'static str, Vec<Property>)>,
>;

unsafe impl<K, V> Send for PersistedPerInstanceHashMap<K, V> {}
unsafe impl<K, V> Sync for PersistedPerInstanceHashMap<K, V> {}

type FnRegisterMap =
  PersistedPerInstanceHashMap<ExportRegisterCallback, (libnode_sys::napi_callback, &'static str)>;
type RegisteredClassesMap = PersistedPerInstanceHashMap<ThreadId, RegisteredClasses>;

static MODULE_REGISTER_CALLBACK: Lazy<ModuleRegisterCallback> = Lazy::new(Default::default);
static MODULE_CLASS_PROPERTIES: Lazy<ModuleClassProperty> = Lazy::new(Default::default);
static IS_FIRST_MODULE: AtomicBool = AtomicBool::new(true);
static FIRST_MODULE_REGISTERED: AtomicBool = AtomicBool::new(false);
static REGISTERED_CLASSES: Lazy<RegisteredClassesMap> = Lazy::new(Default::default);
static FN_REGISTER_MAP: Lazy<FnRegisterMap> = Lazy::new(Default::default);
pub(crate) static CUSTOM_GC_TSFN: std::sync::atomic::AtomicPtr<
  libnode_sys::napi_threadsafe_function__,
> = std::sync::atomic::AtomicPtr::new(ptr::null_mut());
pub(crate) static CUSTOM_GC_TSFN_DESTROYED: AtomicBool = AtomicBool::new(false);
// Store thread id of the thread that created the CustomGC ThreadsafeFunction.
pub(crate) static THREADS_CAN_ACCESS_ENV: once_cell::sync::Lazy<
  PersistedPerInstanceHashMap<ThreadId, bool>,
> = once_cell::sync::Lazy::new(Default::default);

type RegisteredClasses = PersistedPerInstanceHashMap<
  /* export name */ String,
  /* constructor */ libnode_sys::napi_ref,
>;

// compatibility for #[module_exports]
static MODULE_EXPORTS: Lazy<RwLock<Vec<ModuleExportsCallback>>> = Lazy::new(Default::default);

#[inline]
fn wait_first_thread_registered() {
  while !FIRST_MODULE_REGISTERED.load(Ordering::SeqCst) {
    std::hint::spin_loop();
  }
}

#[doc(hidden)]
pub fn get_class_constructor(js_name: &'static str) -> Option<libnode_sys::napi_ref> {
  let current_id = std::thread::current().id();
  REGISTERED_CLASSES.borrow_mut(|map| {
    map
      .get(&current_id)
      .map(|m| m.borrow_mut(|map| map.get(js_name).copied()))
  })?
}

#[doc(hidden)]
// compatibility for #[module_exports]
pub fn register_module_exports(callback: ModuleExportsCallback) {
  MODULE_EXPORTS
    .write()
    .expect("Register module exports failed")
    .push(callback);
}

#[doc(hidden)]
pub fn register_module_export(
  js_mod: Option<&'static str>,
  name: &'static str,
  cb: ExportRegisterCallback,
) {
  MODULE_REGISTER_CALLBACK
    .write()
    .expect("Register module export failed")
    .push((js_mod, (name, cb)));
}

#[doc(hidden)]
pub fn register_js_function(
  name: &'static str,
  cb: ExportRegisterCallback,
  c_fn: libnode_sys::napi_callback,
) {
  FN_REGISTER_MAP.borrow_mut(|inner| {
    inner.insert(cb, (c_fn, name));
  });
}

#[doc(hidden)]
pub fn register_class(
  rust_name: &'static str,
  js_mod: Option<&'static str>,
  js_name: &'static str,
  props: Vec<Property>,
) {
  MODULE_CLASS_PROPERTIES.borrow_mut(|inner| {
    let val = inner.entry(rust_name).or_default();
    let val = val.entry(js_mod).or_default();
    val.0 = js_name;
    val.1.extend(props);
  });
}

#[inline]
/// Get `JsFunction` from defined Rust `fn`
/// ```rust
/// #[napi]
/// fn some_fn() -> u32 {
///     1
/// }
///
/// #[napi]
/// fn return_some_fn() -> Result<JsFunction> {
///     get_js_function(some_fn_js_function)
/// }
/// ```
///
/// ```js
/// returnSomeFn()(); // 1
/// ```
///
pub fn get_js_function(
  env: &Env,
  raw_fn: ExportRegisterCallback,
) -> Result<JsFunction> {
  FN_REGISTER_MAP.borrow_mut(|inner| {
    inner
      .get(&raw_fn)
      .and_then(|(cb, name)| {
        let mut function = ptr::null_mut();
        let name_len = name.len() - 1;
        let fn_name = unsafe { CStr::from_bytes_with_nul_unchecked(name.as_bytes()) };
        check_status!(unsafe {
          libnode_sys::napi_create_function(
            env.0,
            fn_name.as_ptr(),
            name_len as isize,
            *cb,
            ptr::null_mut(),
            &mut function,
          )
        })
        .ok()?;
        Some(JsFunction(Value {
          env: env.0,
          value: function,
          value_type: ValueType::Function,
        }))
      })
      .ok_or_else(|| {
        crate::napi::Error::new(
          crate::napi::Status::InvalidArg,
          "JavaScript function does not exist".to_owned(),
        )
      })
  })
}

/// Get `C Callback` from defined Rust `fn`
/// ```rust
/// #[napi]
/// fn some_fn() -> u32 {
///     1
/// }
///
/// #[napi]
/// fn create_obj(env: Env) -> Result<JsObject> {
///     let mut obj = env.create_object()?;
///     obj.define_property(&[Property::new("getter")?.with_getter(get_c_callback(some_fn_js_function)?)])?;
///     Ok(obj)
/// }
/// ```
///
/// ```js
/// console.log(createObj().getter) // 1
/// ```
///
pub fn get_c_callback(raw_fn: ExportRegisterCallback) -> Result<crate::napi::Callback> {
  FN_REGISTER_MAP.borrow_mut(|inner| {
    inner
      .get(&raw_fn)
      .and_then(|(cb, _name)| *cb)
      .ok_or_else(|| {
        crate::napi::Error::new(
          crate::napi::Status::InvalidArg,
          "JavaScript function does not exist".to_owned(),
        )
      })
  })
}

#[no_mangle]
/// Register the n-api module exports.
///
/// # Safety
/// This method is meant to be called by Node.js while importing the n-api module.
/// Only call this method if the current module is **not** imported by a node-like runtime.
///
/// Arguments `env` and `exports` must **not** be null.
pub unsafe extern "C" fn napi_register_module_v1(
  env: libnode_sys::napi_env,
  exports: libnode_sys::napi_value,
) -> libnode_sys::napi_value {
  if IS_FIRST_MODULE.load(Ordering::SeqCst) {
    IS_FIRST_MODULE.store(false, Ordering::SeqCst);
  } else {
    wait_first_thread_registered();
  }
  let mut exports_objects: HashSet<String> = HashSet::default();

  {
    let mut register_callback = MODULE_REGISTER_CALLBACK
      .write()
      .expect("Write MODULE_REGISTER_CALLBACK in napi_register_module_v1 failed");
    register_callback
      .iter_mut()
      .fold(
        HashMap::<Option<&'static str>, Vec<(&'static str, ExportRegisterCallback)>>::new(),
        |mut acc, (js_mod, item)| {
          if let Some(k) = acc.get_mut(js_mod) {
            k.push(*item);
          } else {
            acc.insert(*js_mod, vec![*item]);
          }
          acc
        },
      )
      .iter()
      .for_each(|(js_mod, items)| {
        let mut exports_js_mod = ptr::null_mut();
        if let Some(js_mod_str) = js_mod {
          let mod_name_c_str =
            unsafe { CStr::from_bytes_with_nul_unchecked(js_mod_str.as_bytes()) };
          if exports_objects.contains(*js_mod_str) {
            check_status_or_throw!(
              env,
              unsafe {
                libnode_sys::napi_get_named_property(
                  env,
                  exports,
                  mod_name_c_str.as_ptr(),
                  &mut exports_js_mod,
                )
              },
              "Get mod {} from exports failed",
              js_mod_str,
            );
          } else {
            check_status_or_throw!(
              env,
              unsafe { libnode_sys::napi_create_object(env, &mut exports_js_mod) },
              "Create export JavaScript Object [{}] failed",
              js_mod_str
            );
            check_status_or_throw!(
              env,
              unsafe {
                libnode_sys::napi_set_named_property(
                  env,
                  exports,
                  mod_name_c_str.as_ptr(),
                  exports_js_mod,
                )
              },
              "Set exports Object [{}] into exports object failed",
              js_mod_str
            );
            exports_objects.insert(js_mod_str.to_string());
          }
        }
        for (name, callback) in items {
          unsafe {
            let js_name = CStr::from_bytes_with_nul_unchecked(name.as_bytes());
            if let Err(e) = callback(env).and_then(|v| {
              let exported_object = if exports_js_mod.is_null() {
                exports
              } else {
                exports_js_mod
              };
              check_status!(
                libnode_sys::napi_set_named_property(env, exported_object, js_name.as_ptr(), v),
                "Failed to register export `{}`",
                name,
              )
            }) {
              JsError::from(e).throw_into(env)
            }
          }
        }
      });
  }

  let mut registered_classes = HashMap::new();

  MODULE_CLASS_PROPERTIES.borrow_mut(|inner| {
    inner.iter().for_each(|(rust_name, js_mods)| {
      for (js_mod, (js_name, props)) in js_mods {
        let mut exports_js_mod = ptr::null_mut();
        unsafe {
          if let Some(js_mod_str) = js_mod {
            let mod_name_c_str = CStr::from_bytes_with_nul_unchecked(js_mod_str.as_bytes());
            if exports_objects.contains(*js_mod_str) {
              check_status_or_throw!(
                env,
                libnode_sys::napi_get_named_property(
                  env,
                  exports,
                  mod_name_c_str.as_ptr(),
                  &mut exports_js_mod,
                ),
                "Get mod {} from exports failed",
                js_mod_str,
              );
            } else {
              check_status_or_throw!(
                env,
                libnode_sys::napi_create_object(env, &mut exports_js_mod),
                "Create export JavaScript Object [{}] failed",
                js_mod_str
              );
              check_status_or_throw!(
                env,
                libnode_sys::napi_set_named_property(
                  env,
                  exports,
                  mod_name_c_str.as_ptr(),
                  exports_js_mod
                ),
                "Set exports Object [{}] into exports object failed",
                js_mod_str
              );
              exports_objects.insert(js_mod_str.to_string());
            }
          }
          let (ctor, props): (Vec<_>, Vec<_>) = props.iter().partition(|prop| prop.is_ctor);

          let ctor = ctor
            .first()
            .map(|c| c.raw().method.unwrap())
            .unwrap_or(noop);
          let raw_props: Vec<_> = props.iter().map(|prop| prop.raw()).collect();

          let js_class_name = CStr::from_bytes_with_nul_unchecked(js_name.as_bytes());
          let mut class_ptr = ptr::null_mut();

          check_status_or_throw!(
            env,
            libnode_sys::napi_define_class(
              env,
              js_class_name.as_ptr(),
              (js_name.len() - 1) as isize,
              Some(ctor),
              ptr::null_mut(),
              raw_props.len(),
              raw_props.as_ptr(),
              &mut class_ptr,
            ),
            "Failed to register class `{}` generate by struct `{}`",
            &js_name,
            &rust_name
          );

          let mut ctor_ref = ptr::null_mut();
          libnode_sys::napi_create_reference(env, class_ptr, 1, &mut ctor_ref);

          registered_classes.insert(js_name.to_string(), ctor_ref);

          check_status_or_throw!(
            env,
            libnode_sys::napi_set_named_property(
              env,
              if exports_js_mod.is_null() {
                exports
              } else {
                exports_js_mod
              },
              js_class_name.as_ptr(),
              class_ptr
            ),
            "Failed to register class `{}` generate by struct `{}`",
            &js_name,
            &rust_name
          );
        }
      }
    });

    REGISTERED_CLASSES.borrow_mut(|map| {
      map.insert(
        std::thread::current().id(),
        PersistedPerInstanceHashMap::from_hashmap(registered_classes),
      )
    });
  });

  {
    let module_exports = MODULE_EXPORTS.read().expect("Read MODULE_EXPORTS failed");
    module_exports.iter().for_each(|callback| unsafe {
      if let Err(e) = callback(env, exports) {
        JsError::from(e).throw_into(env);
      }
    })
  }

  create_custom_gc(env);
  FIRST_MODULE_REGISTERED.store(true, Ordering::SeqCst);
  exports
}

pub(crate) unsafe extern "C" fn noop(
  env: libnode_sys::napi_env,
  _info: libnode_sys::napi_callback_info,
) -> libnode_sys::napi_value {
  if !crate::napi::bindgen_runtime::___CALL_FROM_FACTORY.with(|s| s.load(Ordering::Relaxed)) {
    unsafe {
      libnode_sys::napi_throw_error(
        env,
        ptr::null_mut(),
        c"Class contains no `constructor`, can not new it!".as_ptr(),
      );
    }
  }
  ptr::null_mut()
}

fn create_custom_gc(env: libnode_sys::napi_env) {
  if !FIRST_MODULE_REGISTERED.load(Ordering::SeqCst) {
    let mut custom_gc_fn = ptr::null_mut();
    check_status_or_throw!(
      env,
      unsafe {
        libnode_sys::napi_create_function(
          env,
          "custom_gc".as_ptr().cast(),
          9,
          Some(empty),
          ptr::null_mut(),
          &mut custom_gc_fn,
        )
      },
      "Create Custom GC Function in napi_register_module_v1 failed"
    );
    let mut async_resource_name = ptr::null_mut();
    check_status_or_throw!(
      env,
      unsafe {
        libnode_sys::napi_create_string_utf8(
          env,
          "CustomGC".as_ptr().cast(),
          8,
          &mut async_resource_name,
        )
      },
      "Create async resource string in napi_register_module_v1"
    );
    let mut custom_gc_tsfn = ptr::null_mut();
    check_status_or_throw!(
      env,
      unsafe {
        libnode_sys::napi_create_threadsafe_function(
          env,
          custom_gc_fn,
          ptr::null_mut(),
          async_resource_name,
          0,
          1,
          ptr::null_mut(),
          Some(custom_gc_finalize),
          ptr::null_mut(),
          Some(custom_gc),
          &mut custom_gc_tsfn,
        )
      },
      "Create Custom GC ThreadsafeFunction in napi_register_module_v1 failed"
    );
    check_status_or_throw!(
      env,
      unsafe { libnode_sys::napi_unref_threadsafe_function(env, custom_gc_tsfn) },
      "Unref Custom GC ThreadsafeFunction in napi_register_module_v1 failed"
    );
    CUSTOM_GC_TSFN.store(custom_gc_tsfn, Ordering::Relaxed);
  }

  let current_thread_id = std::thread::current().id();
  THREADS_CAN_ACCESS_ENV.borrow_mut(|m| m.insert(current_thread_id, true));
  check_status_or_throw!(
    env,
    unsafe {
      libnode_sys::napi_add_env_cleanup_hook(
        env,
        Some(remove_thread_id),
        Box::into_raw(Box::new(current_thread_id)).cast(),
      )
    },
    "Failed to add remove thread id cleanup hook"
  );
}

unsafe extern "C" fn remove_thread_id(id: *mut std::ffi::c_void) {
  let thread_id = unsafe { Box::from_raw(id.cast::<ThreadId>()) };
  THREADS_CAN_ACCESS_ENV.borrow_mut(|m| m.insert(*thread_id, false));
}

#[allow(unused)]
unsafe extern "C" fn empty(
  env: libnode_sys::napi_env,
  info: libnode_sys::napi_callback_info,
) -> libnode_sys::napi_value {
  ptr::null_mut()
}

#[allow(unused_variables)]
unsafe extern "C" fn custom_gc_finalize(
  env: libnode_sys::napi_env,
  finalize_data: *mut std::ffi::c_void,
  finalize_hint: *mut std::ffi::c_void,
) {
  CUSTOM_GC_TSFN_DESTROYED.store(true, Ordering::SeqCst);
}

// recycle the ArrayBuffer/Buffer Reference if the ArrayBuffer/Buffer is not dropped on the main thread
extern "C" fn custom_gc(
  env: libnode_sys::napi_env,
  _js_callback: libnode_sys::napi_value,
  _context: *mut std::ffi::c_void,
  data: *mut std::ffi::c_void,
) {
  // current thread was destroyed
  if THREADS_CAN_ACCESS_ENV.borrow_mut(|m| m.get(&std::thread::current().id()) == Some(&false))
    || data.is_null()
  {
    return;
  }
  let mut ref_count = 0;
  check_status_or_throw!(
    env,
    unsafe { libnode_sys::napi_reference_unref(env, data.cast(), &mut ref_count) },
    "Failed to unref Buffer reference in Custom GC"
  );
  debug_assert!(
    ref_count == 0,
    "Buffer reference count in Custom GC is not 0"
  );
  check_status_or_throw!(
    env,
    unsafe { libnode_sys::napi_delete_reference(env, data.cast()) },
    "Failed to delete Buffer reference in Custom GC"
  );
}
