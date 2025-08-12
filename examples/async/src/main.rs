use std::time::Duration;

use edon::napi::JsNumber;
use edon::napi::JsRc;

pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  /*
    interface MyAsyncExtension {
      add_one(a: number): Promise<number>
    }
  */
  nodejs.napi_module_register("my_async_extension", |env, mut exports| {
    let async_fn = env.create_function_from_closure("add_one", |ctx| {
      let arg0 = ctx.get::<JsRc<JsNumber>>(0)?;
      let env = ctx.env.clone();

      ctx.env.spawn_local_promise(async move {
        async_io::Timer::after(Duration::from_secs(1)).await;
        let value = arg0.get_int32()?;
        let js_value = env.create_int32(value + 1)?;

        println!("Rust: Async action complete");

        Ok(js_value)
      })
    })?;

    exports.set_named_property("add_one", async_fn)?;
    Ok(exports)
  })?;

  // Start a new Nodejs context
  let ctx0 = nodejs.spawn_worker_thread()?;

  ctx0.eval(
    r#"
    const native = process._linkedBinding('my_async_extension');

    (async () => {
      const result = await native.add_one(41)
      console.log(result)
    })();
  "#,
  )?;

  Ok(())
}
