pub mod executor;

use std::cell::LazyCell;
use std::cell::OnceCell;
use std::cell::RefCell;
use std::ffi::c_void;
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc::channel;
use std::thread;

use futures::task::LocalSpawnExt;
use libnode_sys as napi_sys;

use self::executor::declare_threadsafe_function::declare_threadsafe_function;
use self::executor::wait_for_wake;
use self::executor::LocalPool;
use self::executor::LocalSpawner;
use self::executor::ThreadNotify;
use self::executor::ThreadNotifyRef;
use crate::napi::Env;

type LocalFuture = Pin<Box<dyn Future<Output = ()>>>;

thread_local! {
  // Custom futures runtime that executes futures on the main thread but can use
  // external threads to wait on futures to pause/resume.
  static LOCAL_POOL: LazyCell<RefCell<LocalPool>> = LazyCell::default();
  static SPAWNER: LazyCell<LocalSpawner> = LazyCell::new(|| LOCAL_POOL.with(|ex| ex.borrow_mut().spawner()));

  // The Nodejs thread safe function used to run futures within
  static EXECUTE_FUTURES: OnceCell<*mut napi_sys::napi_threadsafe_function__> = OnceCell::default();

  // This is a dedicated thread waiting on pending futures to resume.
  // Once they resume it will run the threadsafe function to drive
  // the futures until they complete or pause again.
  static THREAD_NOTIFY: LazyCell<ThreadNotifyRef> = LazyCell::new(|| {
    let (tx_thread_notify, rx_thread_notify) = channel::<ThreadNotifyRef>();
    let tsfn = EXECUTE_FUTURES.with(|v| (*v.get().unwrap()) as usize);

    thread::spawn(move || {
      let thread_notify = ThreadNotify::new();
      tx_thread_notify.send(thread_notify.clone()).unwrap();
      let tsfn = tsfn as *mut napi_sys::napi_threadsafe_function__;

      loop {
        wait_for_wake(&thread_notify);
        unsafe {
          let fut_ptr = Box::into_raw(Box::new(None::<LocalFuture>));
          napi_sys::napi_call_threadsafe_function(tsfn, fut_ptr.cast(), 0);
        }
      }
    });

    rx_thread_notify.recv().unwrap()
  });
}

// This is the callback for the thread safe function used to drive
// the futures forward on the main thread
unsafe extern "C" fn async_runtime_execute(
  env: napi_sys::napi_env,
  _js_callback: napi_sys::napi_value,
  _context: *mut c_void,
  data: *mut c_void,
) {
  let fut_ptr = data.cast::<Option<LocalFuture>>();
  let fut = Box::from_raw(fut_ptr);

  if let Some(fut) = *fut {
    SPAWNER
      .with(move |ls| {
        ls.spawn_local(async move {
          fut.await;
        })
      })
      .expect("Unable to spawn future on local pool");
  }

  let pending_futures = THREAD_NOTIFY.with(|thread_notify| {
    LOCAL_POOL.with(move |lp| {
      let mut lp = lp.borrow_mut();
      lp.run_until_stalled(&thread_notify)
    })
  });

  // If there are no more futures pending then
  // allow the nodejs process to exit
  if pending_futures == 0 {
    EXECUTE_FUTURES.with(|v| {
      let tsfn = *v.get().unwrap();
      napi_sys::napi_unref_threadsafe_function(env, tsfn);
    });
  }
}

#[allow(dead_code)]
pub fn spawn_local<Func, Fut>(
  env: Env,
  fut: Func,
) where
  Func: 'static + Send + FnOnce(Env) -> Fut,
  Fut: Future<Output = ()>,
{
  let env_raw = env.raw();

  // Initialize runtime if not already running
  let tsfn = EXECUTE_FUTURES.with(move |tsfn| {
    *tsfn.get_or_init(move || {
      declare_threadsafe_function(env_raw, "async_runtime_execute", async_runtime_execute)
    })
  });

  // Ensure the thread safe function will prevent Nodejs from exiting until the async task is done
  unsafe { napi_sys::napi_ref_threadsafe_function(env_raw, tsfn) };

  // Pin the future to the current thread and send it to the thread safe function
  let fut = Box::pin(fut(env)) as Pin<Box<dyn Future<Output = ()>>>;
  let fut_ptr = Box::into_raw(Box::new(Some(fut)));
  unsafe { napi_sys::napi_call_threadsafe_function(tsfn, fut_ptr.cast(), 0) };
}

pub fn spawn_local_fut<Fut>(
  env: Env,
  fut: Fut,
) -> crate::napi::Result<()>
where
  Fut: Future<Output = ()>,
{
  let env_raw = env.raw();

  // Initialize runtime if not already running
  let tsfn = EXECUTE_FUTURES.with(move |tsfn| {
    *tsfn.get_or_init(move || {
      declare_threadsafe_function(env_raw, "async_runtime_execute", async_runtime_execute)
    })
  });

  // Ensure the thread safe function will prevent Nodejs from exiting until the async task is done
  unsafe { napi_sys::napi_ref_threadsafe_function(env_raw, tsfn) };

  // Pin the future to the current thread and send it to the thread safe function
  let fut = Box::pin(fut) as Pin<Box<dyn Future<Output = ()>>>;
  let fut_ptr = Box::into_raw(Box::new(Some(fut)));
  unsafe { napi_sys::napi_call_threadsafe_function(tsfn, fut_ptr.cast(), 0) };

  Ok(())
}
