This repository provides embedders to run `dart compile wasm --standalone` modules on
popular WebAssembly runtimes.

- The [runtime directory](./runtime/) contains a Java module suitable for running dart2wasm
  apps on [endive](https://endive.run/).
- [dart2wasmtime](./dart2wasmtime/) runs Dart on [wasmtime](https://wasmtime.dev) with a
  configurable Rust embedder.

To target WASI and the [component model](https://component-model.bytecodealliance.org/), see
[wasm.dart](https://github.com/simolus3/wasm.dart/).

## Running examples

The JVM examples can be run through Gradle, which also compiles Dart to WebAssembly:

```shell
./gradlew examples:hello_world:run

./gradlew examples:swing:run
```

To run a Rust example, compile to WebAssembly first:

```shell
dart compile wasm --standalone dart2wasmtime/examples/hello_world.dart
cargo run --example run -- dart2wasmtime/examples/hello_world.wasm
```
