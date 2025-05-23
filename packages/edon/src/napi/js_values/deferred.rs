use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr;

use libnode_sys;

use crate::napi::bindgen_runtime::ToNapiValue;
use crate::napi::check_status;
use crate::napi::Env;
use crate::napi::Error;
use crate::napi::JsObject;
use crate::napi::Result;
use crate::napi::Value;

struct DeferredData<Data: ToNapiValue, Resolver: FnOnce(Env) -> Result<Data>> {
  resolver: Result<Resolver>,
}

pub struct JsDeferred<Data: ToNapiValue, Resolver: FnOnce(Env) -> Result<Data>> {
  pub(crate) tsfn: libnode_sys::napi_threadsafe_function,
  _data: PhantomData<Data>,
  _resolver: PhantomData<Resolver>,
}

// A trick to send the resolver into the `panic` handler
// Do not use clone in the other place besides the `fn execute_tokio_future`
impl<Data: ToNapiValue, Resolver: FnOnce(Env) -> Result<Data>> Clone
  for JsDeferred<Data, Resolver>
{
  fn clone(&self) -> Self {
    Self {
      tsfn: self.tsfn,
      _data: PhantomData,
      _resolver: PhantomData,
    }
  }
}

unsafe impl<Data: ToNapiValue, Resolver: FnOnce(Env) -> Result<Data>> Send
  for JsDeferred<Data, Resolver>
{
}

impl<Data: ToNapiValue, Resolver: FnOnce(Env) -> Result<Data>> JsDeferred<Data, Resolver> {
  pub(crate) fn new(env: libnode_sys::napi_env) -> Result<(Self, JsObject)> {
    let (tsfn, promise) = js_deferred_new_raw(env, Some(napi_resolve_deferred::<Data, Resolver>))?;

    let deferred = Self {
      tsfn,
      _data: PhantomData,
      _resolver: PhantomData,
    };

    Ok((deferred, promise))
  }

  /// Consumes the deferred, and resolves the promise. The provided function will be called
  /// from the JavaScript thread, and should return the resolved value.
  pub fn resolve(
    self,
    resolver: Resolver,
  ) {
    self.call_tsfn(Ok(resolver))
  }

  /// Consumes the deferred, and rejects the promise with the provided error.
  pub fn reject(
    self,
    error: Error,
  ) {
    self.call_tsfn(Err(error))
  }

  fn call_tsfn(
    self,
    result: Result<Resolver>,
  ) {
    let data = DeferredData { resolver: result };

    // Call back into the JS thread via a threadsafe function. This results in napi_resolve_deferred being called.
    let status = unsafe {
      libnode_sys::napi_call_threadsafe_function(
        self.tsfn,
        Box::into_raw(Box::from(data)).cast(),
        libnode_sys::ThreadsafeFunctionCallMode::blocking,
      )
    };
    debug_assert!(
      status == libnode_sys::Status::napi_ok,
      "Call threadsafe function in JsDeferred failed"
    );

    let status = unsafe {
      libnode_sys::napi_release_threadsafe_function(
        self.tsfn,
        libnode_sys::ThreadsafeFunctionReleaseMode::release,
      )
    };
    debug_assert!(
      status == libnode_sys::Status::napi_ok,
      "Release threadsafe function in JsDeferred failed"
    );
  }
}

fn js_deferred_new_raw(
  env: libnode_sys::napi_env,
  resolve_deferred: libnode_sys::napi_threadsafe_function_call_js,
) -> Result<(libnode_sys::napi_threadsafe_function, JsObject)> {
  let mut raw_promise = ptr::null_mut();
  let mut raw_deferred = ptr::null_mut();
  check_status! {
    unsafe { libnode_sys::napi_create_promise(env, &mut raw_deferred, &mut raw_promise) }
  }?;

  // Create a threadsafe function so we can call back into the JS thread when we are done.
  let mut async_resource_name = ptr::null_mut();
  check_status!(
    unsafe {
      libnode_sys::napi_create_string_utf8(
        env,
        "napi_resolve_deferred\0".as_ptr().cast(),
        22,
        &mut async_resource_name,
      )
    },
    "Create async resource name in JsDeferred failed"
  )?;

  let mut tsfn = ptr::null_mut();
  check_status!(
    unsafe {
      libnode_sys::napi_create_threadsafe_function(
        env,
        ptr::null_mut(),
        ptr::null_mut(),
        async_resource_name,
        0,
        1,
        ptr::null_mut(),
        None,
        raw_deferred.cast(),
        resolve_deferred,
        &mut tsfn,
      )
    },
    "Create threadsafe function in JsDeferred failed"
  )?;

  let promise = JsObject(Value {
    env,
    value: raw_promise,
    value_type: crate::napi::ValueType::Object,
  });

  Ok((tsfn, promise))
}

extern "C" fn napi_resolve_deferred<Data: ToNapiValue, Resolver: FnOnce(Env) -> Result<Data>>(
  env: libnode_sys::napi_env,
  _js_callback: libnode_sys::napi_value,
  context: *mut c_void,
  data: *mut c_void,
) {
  let deferred = context.cast();
  let deferred_data: Box<DeferredData<Data, Resolver>> = unsafe { Box::from_raw(data.cast()) };
  let result = deferred_data
    .resolver
    .and_then(|resolver| resolver(unsafe { Env::from_raw(env) }))
    .and_then(|res| unsafe { ToNapiValue::to_napi_value(env, res) });

  if let Err(e) = result.and_then(|res| {
    check_status!(
      unsafe { libnode_sys::napi_resolve_deferred(env, deferred, res) },
      "Resolve deferred value failed"
    )
  }) {
    let error = Ok::<libnode_sys::napi_value, Error>(unsafe {
      crate::napi::JsError::from(e).into_value(env)
    });

    match error {
      Ok(error) => {
        unsafe { libnode_sys::napi_reject_deferred(env, deferred, error) };
      }
      Err(err) => {
        if cfg!(debug_assertions) {
          println!("Failed to reject deferred: {err:?}");
          let mut err = ptr::null_mut();
          let mut err_msg = ptr::null_mut();
          unsafe {
            libnode_sys::napi_create_string_utf8(
              env,
              "Rejection failed\0".as_ptr().cast(),
              0,
              &mut err_msg,
            );
            libnode_sys::napi_create_error(env, ptr::null_mut(), err_msg, &mut err);
            libnode_sys::napi_reject_deferred(env, deferred, err);
          }
        }
      }
    }
  }
}
