use std::sync::mpsc::channel;

use edon::napi::JsNumber;

pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  let threads = 5;
  let add_until = 100;

  let mut handles = vec![];

  for _ in 0..threads {
    let ctx = nodejs.spawn_context()?;
    handles.push(std::thread::spawn(move || -> anyhow::Result<i32>{
      ctx.eval("globalThis.i = 0;")?;

      // Do addition using eval statements
      for _ in 0..add_until {
        ctx.eval("globalThis.i += 1;")?;
        // ctx.eval("console.log(globalThis.i)")?;
      }

      let (tx, rx) = channel();
      
      // Extract value using native code
      ctx.exec(move |env| {
        let global_this = env.get_global()?;
        let i = global_this.get_named_property::<JsNumber>("i")?;
        tx.send(i.get_int32()?).unwrap();
        Ok(())
      })?;

      Ok(rx.recv()?)
    }));
  }

  let mut i = 0;
  for handle in handles {
    i += handle.join().unwrap()?;
  }
  println!("{}", i);
  assert!(add_until * threads == i, "Invalid sum received");

  Ok(())
}
