use std::{env, path::Path, process::ExitCode};

use dart2wasmtime::{DartEmbedder, add_dart_imports, invoke_main};
use wasmtime::{Config, Engine, Linker, Module, Store};

// cargo run --example run -- /home/simon/src/wasm.dart/playground/hello_world.wasm
pub fn main() -> ExitCode {
    let mut args = env::args_os();
    let program = args.next();
    let (Some(module), None) = (args.next(), args.next()) else {
        let program = program
            .as_deref()
            .map(|p| p.to_string_lossy())
            .unwrap_or("run".into());
        eprintln!("Usage: {program} <module.wasm>");
        return ExitCode::FAILURE;
    };

    match run(module) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e:?}");
            ExitCode::FAILURE
        }
    }
}

fn run(module: impl AsRef<Path>) -> wasmtime::Result<()> {
    let config = Config::new();
    let engine = Engine::new(&config)?;
    let mut store = Store::new(&engine, DemoDartEmbedder);
    let mut linker = Linker::<DemoDartEmbedder>::new(&engine);

    let module = Module::from_file(&engine, module)?;
    add_dart_imports(&mut linker, &module)?;
    let instance = linker.instantiate(&mut store, &module)?;
    println!("Instantiated Dart app!");

    invoke_main(&instance, &mut store)?;
    Ok(())
}

struct DemoDartEmbedder;

impl DartEmbedder for DemoDartEmbedder {
    fn print(&self, msg: &str) -> wasmtime::Result<()> {
        println!("{}", msg);
        Ok(())
    }
}
