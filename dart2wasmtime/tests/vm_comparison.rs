use std::{
    cell::RefCell,
    collections::VecDeque,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use dart2wasmtime::{DartCallback, DartEmbedder, DartSchedule, add_dart_imports, invoke_main};
use wasmtime::{Config, Engine, Linker, Module, Result, Store, error::Context, format_err};

#[test]
fn matches_dartvm_output() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let goldens = root.join("goldens");
    let out_dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("vm_comparison");
    fs::create_dir_all(&out_dir).unwrap();

    let mut programs = fs::read_dir(root.join("vm_comparison"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "dart"))
        .collect::<Vec<_>>();
    programs.sort();
    assert!(!programs.is_empty(), "No test programs found");

    let mut failures = Vec::new();
    for program in &programs {
        let name = program.file_name().unwrap().to_string_lossy().into_owned();
        let golden = goldens.join(format!("{name}.txt"));
        let Ok(expected) = fs::read_to_string(&golden) else {
            failures.push(format!(
                "{name}: missing golden at {}, run `dart tool/update_goldens.dart`",
                golden.display()
            ));
            continue;
        };

        match compile_and_run(program, &out_dir) {
            Ok(actual) if actual == expected => {}
            Ok(actual) => failures.push(format!(
                "{name}: output mismatch\n--- expected (VM)\n{expected}\n--- actual (dart2wasmtime)\n{actual}"
            )),
            Err(e) => failures.push(format!("{name}: {e:?}")),
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} programs failed:\n\n{}",
        failures.len(),
        programs.len(),
        failures.join("\n\n")
    );
}

fn compile_and_run(program: &Path, out_dir: &Path) -> Result<String> {
    let wasm = compile(program, out_dir)?;

    let engine = Engine::new(&Config::new())?;
    let module = Module::from_file(&engine, &wasm)?;
    let mut linker = Linker::<TestEmbedder>::new(&engine);
    add_dart_imports(&mut linker, &module)?;

    let mut store = Store::new(&engine, TestEmbedder::default());
    let instance = linker.instantiate(&mut store, &module)?;

    invoke_main(&instance, &mut store).context("Running main")?;
    drain_microtasks(&mut store)?;

    Ok(store.into_data().output.into_inner())
}

fn compile(program: &Path, out_dir: &Path) -> Result<PathBuf> {
    let name = program.file_stem().unwrap().to_string_lossy();
    let wasm = out_dir.join(format!("{name}.wasm"));

    let result = Command::new("dart")
        .args(["compile", "wasm", "--standalone", "-o"])
        .arg(&wasm)
        .arg(program)
        .arg("-E--no-strip-wasm")
        .output()
        .context("Running dart")?;
    if !result.status.success() {
        return Err(format_err!(
            "dart compile wasm failed ({}):\n{}{}",
            result.status,
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        ));
    }

    Ok(wasm)
}

fn drain_microtasks(store: &mut Store<TestEmbedder>) -> Result<()> {
    while let Some(microtask) = store.data_mut().microtasks.pop_front() {
        microtask.invoke(&mut *store).context("Running microtask")?;
    }
    Ok(())
}

#[derive(Default)]
struct TestEmbedder {
    /// Lines printed by the Dart program.
    output: RefCell<String>,
    /// Scheduled microtasks.
    microtasks: VecDeque<DartCallback>,
}

impl DartEmbedder for TestEmbedder {
    type Timer = NopTimer;

    fn print(&self, msg: &str) -> Result<()> {
        let mut output = self.output.borrow_mut();
        output.push_str(msg);
        output.push('\n');
        Ok(())
    }

    fn queue_microtask(&mut self, callback: DartCallback) -> Result<()> {
        self.microtasks.push_back(callback);
        Ok(())
    }

    fn random_int(&mut self, _secure: bool) -> Result<i64> {
        // chosen by fair dice roll, guaranteed to be random
        Ok(4)
    }
}

struct NopTimer;

impl DartSchedule for NopTimer {
    fn clear_schedule(&mut self) {}
}
