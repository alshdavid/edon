use futures::Future;
use napi::Env;
use napi::JsObject;
use napi::NapiValue;

use crate::spawn_local;
use crate::spawn_local_promise;

pub trait SpawnLocalExt {
  /// Spawns a non-blocking future on the local thread.
  /// Normal [`NapiValue`] types can be interacted with in
  /// the async context. Supports channels, timers, etc.
  ///
  /// Equivalent to:
  ///
  /// ```javascript
  /// setTimeout(async () => { await work() }, 0)
  /// ```
  ///
  /// To ensure the availability of [`NapiValue`] types beyond the life of the parent function scope,
  /// ensure that [`NapiValue`] types that will be used in an async closure are wrapped in a [`crate::JsRc`].
  ///
  /// ### Usage:
  ///
  /// #### Running a Callback:
  ///
  /// ```
  /// use std::time::Duration;
  ///
  /// use napi::*;
  /// use napi_derive::napi;
  /// use napi_async_local::JsRc;
  /// use napi_async_local::JsRcExt;
  /// use napi_async_local::SpawnLocalExt;
  /// use async_std::task;
  ///
  /// #[napi]
  /// fn my_js_func(env: Env, callback: JsRc<JsFunction>) -> napi::Result<()> {
  ///   env.spawn_local(move |env| async move {
  ///     task::sleep(Duration::from_millis(1000)).await;
  ///     callback.inner(&env)?.call_without_args(None)?;
  ///     Ok(())
  ///   })
  /// }
  /// ```
  ///
  /// #### Using Channels:
  ///
  /// ```
  /// use std::thread;
  /// use std::time::Duration;
  ///
  /// use napi::*;
  /// use napi_derive::napi;
  /// use napi_async_local::JsRc;
  /// use napi_async_local::JsRcExt;
  /// use napi_async_local::SpawnLocalExt;
  /// use async_std::channel;
  ///
  /// #[napi]
  /// fn my_js_func(env: Env, callback: JsRc<JsFunction>) -> napi::Result<()> {
  ///   let (tx, rx) = channel::unbounded();
  ///
  ///   thread::spawn(move || {
  ///     for i in 0..10 {
  ///       tx.send_blocking(i).unwrap();
  ///       thread::sleep(Duration::from_millis(1000));
  ///     }
  ///   });
  ///
  ///   env.spawn_local(move |env| async move {
  ///     while let Ok(value) = rx.recv().await {
  ///       println!("Got number: {}", value);
  ///       callback.inner(&env)?.call(None, &[env.create_int32(value)?])?;
  ///     }
  ///
  ///     Ok(())
  ///   })
  /// }
  /// ```
  fn spawn_local<Fut>(
    &self,
    future: Fut,
  ) -> napi::Result<()>
  where
    Fut: Future<Output = napi::Result<()>> + 'static;

  /// Spawns a non-blocking future on the local thread. Returns a Promise with the value
  /// returned in the async closure. Normal [`NapiValue`] types can be interacted with in
  /// the async context. Supports channels, timers, etc.
  ///
  /// Equivalent to:
  ///
  /// ```javascript
  /// new Promise((res, rej) => setTimeout(async () => {
  ///   try {
  ///     res(await work())
  ///   } catch(err) {
  ///     rej(err)
  ///   }
  /// }, 0)
  /// ```
  ///
  /// To ensure the availability of [`NapiValue`] types beyond the life of the parent function scope,
  /// ensure that [`NapiValue`] types that will be used in an async closure are wrapped in a [`crate::JsRc`].
  ///
  /// ### Usage:
  ///
  /// ```
  /// use std::time::Duration;
  ///
  /// use napi::*;
  /// use napi_derive::napi;
  /// use napi_async_local::SpawnLocalExt;
  /// use async_std::task;
  ///
  /// #[napi]
  /// fn my_js_func(env: Env) -> napi::Result<JsObject> {
  ///   env.spawn_local_promise(move |env| async move {
  ///     task::sleep(Duration::from_millis(1000)).await;
  ///     env.create_string("Hello World")
  ///   })
  /// }
  /// ```
  fn spawn_local_promise<R, Fut>(
    &self,
    future: Fut,
  ) -> napi::Result<JsObject>
  where
    R: NapiValue + 'static,
    Fut: Future<Output = napi::Result<R>> + 'static;
}

impl SpawnLocalExt for Env {
  fn spawn_local<Fut>(
    &self,
    future: Fut,
  ) -> napi::Result<()>
  where
    Fut: Future<Output = napi::Result<()>> + 'static,
  {
    spawn_local(self, future)
  }

  fn spawn_local_promise<R, Fut>(
    &self,
    future: Fut,
  ) -> napi::Result<JsObject>
  where
    R: NapiValue + 'static,
    Fut: Future<Output = napi::Result<R>> + 'static,
  {
    spawn_local_promise(self, future)
  }
}
