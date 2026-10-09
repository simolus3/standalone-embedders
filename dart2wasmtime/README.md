# dart2wasmtime

This crate implements a `dart2wasm` [standalone embedder] with `wasmtime`, allowing you to load
compiled Dart programs as modules into a Rust library or application.

## Usage

To use this, compile a Dart program with `dart compile wasm --standalone`. The resulting `.wasm` file
can be instantiated through `dart2wasmtime`.

To embed Dart, implement the `dart2wasmtime::DartEmbedder` trait as the `data`
for an `wasmtime::Store`. Then, use `dart2wasmtime::add_dart_imports` to add
imports needed to run the Dart program:

```rs
struct MyDartEmbedder;
struct TodoSchedule;

impl DartEmbedder for MyDartEmbedder {
    type Timer = TodoSchedule;

    fn print(&self, msg: &str) -> wasmtime::Result<()> {
        println!("{}", msg);
        Ok(())
    }
}

impl DartSchedule for TodoSchedule {
    fn clear_schedule(&mut self) {
        // Only needed when implementing an event loop
    }
}

fn load_dart() -> wasmtime::Result<()> {
    let config = Config::new();
    let engine = Engine::new(&config)?;
    let mut store = Store::new(&engine, MyDartEmbedder);
    let mut linker = Linker::<MyDartEmbedder>::new(&engine);

    let module = Module::from_file(&engine, ...)?;
    add_dart_imports(&mut linker, &module)?;
    let instance = linker.instantiate(&mut store, &module)?;

    invoke_main(&instance, &mut store)?;
    Ok(())
}
```

For a fully-featured Dart embedder, see [run.rs](./examples/run.rs).

## Advanced

Dart code can import additional functions that can be implemented in Rust (and
added to the linker as usual).
Dart can export extra functions as well, which can then be called from Rust.

For more information, see [imports and exports].

[standalone embedder]: https://github.com/dart-lang/sdk/blob/main/pkg/dart2wasm/docs/standalone.md
[imports and exports]: https://github.com/dart-lang/sdk/blob/main/pkg/dart2wasm/docs/imports_and_exports.md
