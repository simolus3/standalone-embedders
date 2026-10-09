use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Instant, SystemTime},
};

use clap::Parser;
use dart2wasmtime::{
    DartCallback, DartEmbedder, DartSchedule, StopwatchFrequency, add_dart_imports, invoke_main,
};
use jiff::{Timestamp, tz::TimeZone};
use rand::Rng;
use tokio::{
    spawn,
    sync::mpsc::{self, Receiver, Sender, WeakSender},
    task::JoinHandle,
    time::sleep,
};
use wasmtime::{
    Config, Engine, Instance, Linker, Module, Result, Store, error::Context, format_err,
};

/// Runs a Dart module compiled with dart2wasm.
#[derive(Parser)]
struct Args {
    /// Path to the compiled `.wasm` module.
    module: PathBuf,
    /// Only instantiate the module without invoking `main`.
    #[arg(long)]
    instantiate_only: bool,
}

// Run with cargo run --example run <path to wasm>
#[tokio::main(flavor = "current_thread")]
pub async fn main() -> ExitCode {
    let args = Args::parse();

    let (sender, receiver) = mpsc::channel(1);

    let (store, instance) = match instantiate(&args.module, sender.clone()) {
        Ok(ok) => ok,
        Err(e) => {
            eprintln!("Error: {e:?}");
            return ExitCode::FAILURE;
        }
    };

    if args.instantiate_only {
        return ExitCode::SUCCESS;
    }

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
            stopwatch_epoch: None,
            time_zone: TimeZone::system(),
        },
    );
    let mut linker = Linker::<DemoDartEmbedder>::new(&engine);

    let module = Module::from_file(&engine, module)?;
    add_dart_imports(&mut linker, &module)?;
    let instance = linker.instantiate(&mut store, &module)?;

    Ok((store, instance))
}

async fn run_dart_app(
    mut store: Store<DemoDartEmbedder>,
    instance: Instance,
    mut receiver: Receiver<DartEvent>,
) -> Result<()> {
    while let Some(event) = receiver.recv().await {
        match event {
            DartEvent::Start { .. } => invoke_main(&instance, &mut store, &[]),
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
    stopwatch_epoch: Option<Instant>,
    time_zone: TimeZone,
}

impl DemoDartEmbedder {
    fn time_zone_info(&self, seconds_since_epoch: i64) -> Result<jiff::tz::TimeZoneOffsetInfo<'_>> {
        let timestamp = Timestamp::from_second(seconds_since_epoch)?;
        Ok(self.time_zone.to_offset_info(timestamp))
    }

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

    fn random_int(&mut self, _secure: bool) -> Result<i64> {
        let mut rng = rand::rng();
        Ok(rng.next_u64() as i64)
    }

    fn monotonic_ticks(&mut self) -> Result<i64> {
        let now = Instant::now();
        let epoch = self.stopwatch_epoch.get_or_insert(now);

        Ok(epoch.duration_since(now).as_micros() as i64)
    }

    fn time_zone_name(&self, seconds_since_epoch: i64) -> Result<String> {
        Ok(self
            .time_zone_info(seconds_since_epoch)?
            .abbreviation()
            .to_string())
    }

    fn time_zone_offset_in_seconds(&self, seconds_since_epoch: i64) -> Result<i32> {
        Ok(self.time_zone_info(seconds_since_epoch)?.offset().seconds())
    }

    const MONOTONIC_FREQUENCY: StopwatchFrequency = StopwatchFrequency::MegaHertz;
}

struct DemoSchedule(JoinHandle<()>);

impl DartSchedule for DemoSchedule {
    fn clear_schedule(&mut self) {
        self.0.abort();
    }
}
