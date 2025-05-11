use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;

use libnode_sys;

use crate::napi::bindgen_runtime::ToNapiValue;
use crate::napi::check_status;
use crate::napi::js_values::NapiValue;
use crate::napi::Env;
use crate::napi::JsError;
use crate::napi::JsObject;
use crate::napi::Result;
use crate::napi::Task;

struct AsyncWork<T: Task> {
  inner_task: T,
  deferred: libnode_sys::napi_deferred,
  value: Result<mem::MaybeUninit<T::Output>>,
  napi_async_work: libnode_sys::napi_async_work,
  status: Rc<AtomicU8>,
}

pub struct AsyncWorkPromise {
  pub(crate) napi_async_work: libnode_sys::napi_async_work,
  raw_promise: libnode_sys::napi_value,
  pub(crate) deferred: libnode_sys::napi_deferred,
  env: libnode_sys::napi_env,
  /// share with AsyncWork
  /// 0: not started
  /// 1: completed
  /// 2: canceled
  pub(crate) status: Rc<AtomicU8>,
}

impl AsyncWorkPromise {
  pub fn promise_object(&self) -> JsObject {
    unsafe { JsObject::from_raw_unchecked(self.env, self.raw_promise) }
  }

  pub fn cancel(&self) -> Result<()> {
    // must be happened in the main thread, relaxed is enough
    self.status.store(2, Ordering::Relaxed);
    check_status!(unsafe { libnode_sys::napi_cancel_async_work(self.env, self.napi_async_work) })
  }
}

pub fn run<T: Task>(
  env: libnode_sys::napi_env,
  task: T,
  abort_status: Option<Rc<AtomicU8>>,
) -> Result<AsyncWorkPromise> {
  let mut raw_resource = ptr::null_mut();
  check_status!(unsafe { libnode_sys::napi_create_object(env, &mut raw_resource) })?;
  let mut raw_promise = ptr::null_mut();
  let mut deferred = ptr::null_mut();
  check_status!(unsafe { libnode_sys::napi_create_promise(env, &mut deferred, &mut raw_promise) })?;
  let task_status = abort_status.unwrap_or_else(|| Rc::new(AtomicU8::new(0)));
  let result = Box::leak(Box::new(AsyncWork {
    inner_task: task,
    deferred,
    value: Ok(mem::MaybeUninit::zeroed()),
    napi_async_work: ptr::null_mut(),
    status: task_status.clone(),
  }));
  let mut async_work_name = ptr::null_mut();
  let s = "napi_rs_async_work";
  let len = s.len();
  let s = CString::new(s)?;
  check_status!(unsafe {
    libnode_sys::napi_create_string_utf8(env, s.as_ptr(), len as isize, &mut async_work_name)
  })?;
  check_status!(unsafe {
    libnode_sys::napi_create_async_work(
      env,
      raw_resource,
      async_work_name,
      Some(execute::<T>),
      Some(complete::<T>),
      (result as *mut AsyncWork<T>).cast(),
      &mut result.napi_async_work,
    )
  })?;
  check_status!(unsafe { libnode_sys::napi_queue_async_work(env, result.napi_async_work) })?;
  Ok(AsyncWorkPromise {
    napi_async_work: result.napi_async_work,
    raw_promise,
    deferred,
    env,
    status: task_status,
  })
}

unsafe impl<T: Task + Send> Send for AsyncWork<T> {}
unsafe impl<T: Task + Sync> Sync for AsyncWork<T> {}

/// env here is the same with the one in `CallContext`.
/// So it actually could do nothing here, because `execute` function is called in the other thread mostly.
unsafe extern "C" fn execute<T: Task>(
  _env: libnode_sys::napi_env,
  data: *mut c_void,
) {
  let mut work = unsafe { Box::from_raw(data as *mut AsyncWork<T>) };
  let _ = mem::replace(
    &mut work.value,
    work.inner_task.compute().map(mem::MaybeUninit::new),
  );
  Box::leak(work);
}

unsafe extern "C" fn complete<T: Task>(
  env: libnode_sys::napi_env,
  status: libnode_sys::napi_status,
  data: *mut c_void,
) {
  let mut work = unsafe { Box::from_raw(data as *mut AsyncWork<T>) };
  let value_ptr = mem::replace(&mut work.value, Ok(mem::MaybeUninit::zeroed()));
  let deferred = mem::replace(&mut work.deferred, ptr::null_mut());
  let napi_async_work = mem::replace(&mut work.napi_async_work, ptr::null_mut());
  let value = match value_ptr {
    Ok(v) => {
      let output = unsafe { v.assume_init() };
      work
        .inner_task
        .resolve(unsafe { Env::from_raw(env) }, output)
    }
    Err(e) => work.inner_task.reject(unsafe { Env::from_raw(env) }, e),
  };
  if status != libnode_sys::Status::napi_cancelled && work.status.load(Ordering::Relaxed) != 2 {
    match check_status!(status)
      .and_then(move |_| value)
      .and_then(|v| unsafe { ToNapiValue::to_napi_value(env, v) })
    {
      Ok(v) => {
        let status = unsafe { libnode_sys::napi_resolve_deferred(env, deferred, v) };
        debug_assert!(
          status == libnode_sys::Status::napi_ok,
          "Resolve promise failed, status: {:?}",
          crate::napi::Status::from(status)
        );
      }
      Err(e) => {
        let status = unsafe {
          libnode_sys::napi_reject_deferred(env, deferred, JsError::from(e).into_value(env))
        };
        debug_assert!(
          status == libnode_sys::Status::napi_ok,
          "Reject promise failed, status: {:?}",
          crate::napi::Status::from(status)
        );
      }
    };
  }
  if let Err(e) = work.inner_task.finally(unsafe { Env::from_raw(env) }) {
    debug_assert!(false, "Panic in Task finally fn: {e:?}");
  }
  let delete_status = unsafe { libnode_sys::napi_delete_async_work(env, napi_async_work) };
  debug_assert!(
    delete_status == libnode_sys::Status::napi_ok,
    "Delete async work failed, status {:?}",
    crate::napi::Status::from(delete_status)
  );
  work.status.store(1, Ordering::Relaxed);
}
