// Usage
// cargo run --package edon_examples -- eval_main

mod basic;
mod eval_main;
mod eval_workers;
mod multiple_contexts;
mod multiple_contexts_load_balance;
mod native_exec;
mod native_module;

fn main() -> anyhow::Result<()> {
  let example = std::env::args()
    .collect::<Vec<String>>()
    .get(1)
    .cloned()
    .unwrap_or("basic".to_string());

  match example.as_str() {
    "basic" => basic::main(),
    "eval_main" => eval_main::main(),
    "eval_workers" => eval_workers::main(),
    "multiple_contexts" => multiple_contexts::main(),
    "multiple_contexts_load_balance" => multiple_contexts_load_balance::main(),
    "native_exec" => native_exec::main(),
    "native_module" => native_module::main(),
    _ => Err(anyhow::anyhow!("No example for: \"{}\"", example)),
  }
}
