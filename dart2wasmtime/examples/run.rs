use std::{collections::VecDeque, env, path::Path, process::ExitCode, time::SystemTime};

use dart2wasmtime::{DartCallback, DartEmbedder, DartSchedule, add_dart_imports, invoke_main};
use tokio::{
    spawn,
    sync::mpsc::{self, Receiver, Sender, WeakSender},
    task::JoinHandle,
    time::sleep,
};
use wasmtime::{
    Config, Engine, Instance, Linker, Module, Result, Store, error::Context, format_err,
};

// cargo run --example run -- /home/simon/src/wasm.dart/playground/hello_world.wasm
#[tokio::main(flavor = "current_thread")]
pub async fn main() -> ExitCode {
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

    let (sender, receiver) = mpsc::channel(1);

    let (store, instance) = match instantiate(module, sender.clone()) {
        Ok(ok) => ok,
        Err(e) => {
            eprintln!("Error: {e:?}");
            return ExitCode::FAILURE;
        }
    };

    let task = spawn(run_dart_app(store, instance, receiver));
    let _ = sender
        .send(DartEvent::Start {
            _keepalive: sender.clone(),
        })
        .await;
    drop(sender);

    match task.await.unwrap() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error running Dart app: {e:?}");
            ExitCode::FAILURE
        }
    }
}

fn instantiate(
    module: impl AsRef<Path>,
    sender: Sender<DartEvent>,
) -> wasmtime::Result<(Store<DemoDartEmbedder>, Instance)> {
    let config = Config::new();
    let engine = Engine::new(&config)?;
    let mut store = Store::new(
        &engine,
        DemoDartEmbedder {
            sender: sender.downgrade(),
            microtasks: Default::default(),
        },
    );
    let mut linker = Linker::<DemoDartEmbedder>::new(&engine);

    let module = Module::from_file(&engine, module)?;
    add_dart_imports(&mut linker, &module)?;
    let instance = linker.instantiate(&mut store, &module)?;
    println!("Instantiated Dart app!");

    Ok((store, instance))
}

async fn run_dart_app(
    mut store: Store<DemoDartEmbedder>,
    instance: Instance,
    mut receiver: Receiver<DartEvent>,
) -> Result<()> {
    while let Some(event) = receiver.recv().await {
        match event {
            DartEvent::Start { .. } => invoke_main(&instance, &mut store),
            DartEvent::Timer(ref callback) => callback.invoke(&mut store),
        }
        .with_context(|| format!("Handling event {event:?}"))?;

        while let Some(microtask) = store.data_mut().microtasks.pop_back() {
            microtask
                .invoke(&mut store)
                .with_context(|| format!("Handling microtask after event {event:?}"))?;
        }
    }

    Ok(())
}

#[derive(Debug)]
enum DartEvent {
    Start { _keepalive: Sender<DartEvent> },
    Timer(DartCallback),
}

struct DemoDartEmbedder {
    sender: WeakSender<DartEvent>,
    microtasks: VecDeque<DartCallback>,
}

impl DemoDartEmbedder {
    fn obtain_sender(&self) -> Result<Sender<DartEvent>> {
        self.sender
            .upgrade()
            .ok_or_else(|| format_err!("Sender already dropped"))
    }
}

impl DartEmbedder for DemoDartEmbedder {
    type Timer = DemoSchedule;

    fn print(&self, msg: &str) -> wasmtime::Result<()> {
        println!("{}", msg);
        Ok(())
    }

    fn schedule_once(
        &mut self,
        duration: std::time::Duration,
        callback: DartCallback,
    ) -> Result<Self::Timer> {
        let delay = sleep(duration);
        let sender = self.obtain_sender()?;

        let handle = spawn(async move {
            delay.await;
            let _ = sender.send(DartEvent::Timer(callback)).await;
        });

        Ok(DemoSchedule(handle))
    }

    fn schedule_repeated(
        &mut self,
        duration: std::time::Duration,
        callback: DartCallback,
    ) -> Result<Self::Timer> {
        let sender = self.obtain_sender()?;

        let handle = spawn(async move {
            loop {
                sleep(duration).await;
                let Ok(()) = sender.send(DartEvent::Timer(callback.clone())).await else {
                    break;
                };
            }
        });

        Ok(DemoSchedule(handle))
    }

    fn queue_microtask(&mut self, callback: DartCallback) -> Result<()> {
        self.microtasks.push_back(callback);
        Ok(())
    }

    fn unix_timestamp(&mut self) -> Result<std::time::Duration> {
        let now = SystemTime::now();
        now.duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| format_err!("time before unix epoch??"))
    }
}

struct DemoSchedule(JoinHandle<()>);

impl DartSchedule for DemoSchedule {
    fn clear_schedule(&mut self) {
        self.0.abort();
    }
}
