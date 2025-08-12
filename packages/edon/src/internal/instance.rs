use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use super::JsTransferable;
use crate::napi::threadsafe_function::ErrorStrategy;
use crate::napi::threadsafe_function::ThreadsafeFunctionCallMode;
use crate::napi::JsFunction;
use crate::napi::JsString;
use crate::napi::JsUnknown;
use crate::Env;

static STARTED: AtomicBool = AtomicBool::new(false);

pub enum NodejsMainEvent {
  StartCommonjsWorker {
    rx_wrk: Receiver<NodejsWorkerEvent>,
    argv: Vec<String>,
    resolve: Sender<String>,
  },
  StopCommonjsWorker {
    id: String,
    resolve: Sender<()>,
  },
  StopMain {
    resolve: Sender<()>,
  },
}

pub enum NodejsWorkerEvent {
  Exec {
    callback: Box<dyn Send + FnOnce(Env) -> crate::Result<()>>,
    resolve: Sender<crate::Result<()>>,
  },
  Eval {
    code: String,
    resolve: Sender<crate::Result<()>>,
  },
  EvalModule {
    code: String,
    resolve: Sender<crate::Result<()>>,
  },
  Require {
    specifier: String,
    resolve: Sender<crate::Result<()>>,
  },
  Import {
    specifier: String,
    resolve: Sender<crate::Result<()>>,
  },
}

pub fn start_node_instance() -> crate::Result<Sender<NodejsMainEvent>> {
  if STARTED
    .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
    .is_err()
  {
    return Err(crate::Error::NodejsAlreadyRunning);
  };

  let (tx, rx) = channel();
  let rx = Arc::new(Mutex::new(Some(rx)));

  super::napi_module_register("edon:main", move |env, mut exports| {
    let js_on_event = env.create_function_from_closure("edon::main::onEvent", {
      let rx = rx.clone();
      move |ctx| {
        let callback = ctx.get::<JsFunction>(0)?;

        let on_eval = callback
          .create_threadsafe_function::<NodejsMainEvent, JsUnknown, _, ErrorStrategy::Fatal>(
            0,
            move |ctx| match ctx.value {
              NodejsMainEvent::StartCommonjsWorker {
                rx_wrk,
                argv,
                resolve,
              } => {
                let action = ctx.env.create_uint32(0)?.into_unknown();

                // [argv, tx_worker]
                let mut payload = ctx.env.create_array(2)?;

                let mut argv_js = ctx.env.create_array(0)?;
                for (i, v) in argv.iter().enumerate() {
                  argv_js.set(i as u32, ctx.env.create_string(v)?)?;
                }

                payload.set(0, argv_js)?;
                payload.set(
                  1,
                  JsTransferable::new(Mutex::new(Some(rx_wrk))).into_unknown(&ctx.env)?,
                )?;
                let payload = payload.coerce_to_object()?.into_unknown();

                let resolve = ctx
                  .env
                  .create_function_from_closure("NodejsEvent::done", move |ctx| {
                    let id = ctx.get::<JsString>(0)?;
                    resolve.send(id.into_utf8()?.as_str()?.to_string()).unwrap();
                    ctx.env.get_undefined()
                  })?
                  .into_unknown();

                Ok(vec![action, payload, resolve])
              }
              NodejsMainEvent::StopCommonjsWorker { id, resolve } => {
                let action = ctx.env.create_uint32(1)?.into_unknown();
                let payload = ctx.env.create_string(&id)?.into_unknown();
                let resolve = ctx
                  .env
                  .create_function_from_closure("NodejsEvent::done", move |ctx| {
                    resolve.send(()).unwrap();
                    ctx.env.get_undefined()
                  })?
                  .into_unknown();

                Ok(vec![action, payload, resolve])
              }
              NodejsMainEvent::StopMain { resolve } => {
                let action = ctx.env.create_uint32(2)?.into_unknown();
                let payload = ctx.env.get_undefined()?.into_unknown();
                let resolve = ctx
                  .env
                  .create_function_from_closure("NodejsEvent::done", move |ctx| {
                    resolve.send(()).unwrap();
                    ctx.env.get_undefined()
                  })?
                  .into_unknown();

                Ok(vec![action, payload, resolve])
              }
            },
          )?;

        thread::spawn({
          let rx = rx.clone();
          move || {
            let Some(rx) = rx.lock().unwrap().take() else {
              panic!("Cannot run twice")
            };

            while let Ok(event) = rx.recv() {
              on_eval.call(event, ThreadsafeFunctionCallMode::Blocking);
            }
          }
        });

        Ok(())
      }
    });

    exports.set_named_property("onEvent", js_on_event)?;

    Ok(exports)
  })?;

  super::napi_module_register("edon:worker", move |env, mut exports| {
    let js_on_event = env.create_function_from_closure("edon::main::onEvent", |ctx| {
      let callback = ctx.get::<JsFunction>(1)?;

      let on_eval = callback
        .create_threadsafe_function::<NodejsWorkerEvent, JsUnknown, _, ErrorStrategy::Fatal>(
          0,
          move |ctx| match ctx.value {
            NodejsWorkerEvent::Exec { callback, resolve } => {
              resolve.send(callback(ctx.env)).ok();
              Ok(vec![])
            }
            NodejsWorkerEvent::Eval { code, resolve } => {
              let action = ctx.env.create_uint32(0)?.into_unknown();
              let payload = ctx.env.create_string(&code)?.into_unknown();
              let resolve = ctx
                .env
                .create_function_from_closure("NodejsContextEvent::done", move |ctx| {
                  resolve.send(Ok(())).unwrap();
                  ctx.env.get_undefined()
                })?
                .into_unknown();

              Ok(vec![action, payload, resolve])
            }
            NodejsWorkerEvent::EvalModule { code, resolve } => {
              let action = ctx.env.create_uint32(1)?.into_unknown();
              let payload = ctx.env.create_string(&code)?.into_unknown();
              let resolve = ctx
                .env
                .create_function_from_closure("NodejsContextEvent::done", move |ctx| {
                  resolve.send(Ok(())).unwrap();
                  ctx.env.get_undefined()
                })?
                .into_unknown();

              Ok(vec![action, payload, resolve])
            }
            NodejsWorkerEvent::Require { specifier, resolve } => {
              let action = ctx.env.create_uint32(2)?.into_unknown();
              let payload = ctx.env.create_string(&specifier)?.into_unknown();
              let resolve = ctx
                .env
                .create_function_from_closure("NodejsContextEvent::done", move |ctx| {
                  resolve.send(Ok(())).unwrap();
                  ctx.env.get_undefined()
                })?
                .into_unknown();

              Ok(vec![action, payload, resolve])
            }
            NodejsWorkerEvent::Import { specifier, resolve } => {
              let action = ctx.env.create_uint32(3)?.into_unknown();
              let payload = ctx.env.create_string(&specifier)?.into_unknown();
              let resolve = ctx
                .env
                .create_function_from_closure("NodejsContextEvent::done", move |ctx| {
                  resolve.send(Ok(())).unwrap();
                  ctx.env.get_undefined()
                })?
                .into_unknown();

              Ok(vec![action, payload, resolve])
            }
          },
        )?;

      let rx = ctx.get::<JsTransferable<Mutex<Option<Receiver<NodejsWorkerEvent>>>>>(0)?;
      let rx = rx.take()?.lock().unwrap().take().unwrap();

      thread::spawn({
        move || {
          while let Ok(event) = rx.recv() {
            on_eval.call(event, ThreadsafeFunctionCallMode::Blocking);
          }
        }
      });
      Ok(())
    });

    exports.set_named_property("onEvent", js_on_event)?;

    Ok(exports)
  })?;

  std::thread::spawn(move || {
    super::eval_blocking(format!("{};\n", crate::prelude::MAIN_JS)).unwrap();
  });

  Ok(tx)
}
