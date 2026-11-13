use core::{any::Any, time::Duration};

use wasmtime::{
    AnyRef, AsContextMut, Caller, ExternRef, Func, OwnedRooted, Result, Rooted, Val, format_err,
};

use crate::{DartEmbedder, utils::externref_mut};

/// Extracts a [DartCallback] from a `(callback, arg)` pair of arguments passed to a `func_new`
/// host function.
fn callback_from_args<E: DartEmbedder>(
    caller: &mut Caller<'_, E>,
    callback: &Val,
    arg: &Val,
) -> Result<DartCallback> {
    let callback = *callback
        .func_ref()
        .flatten()
        .ok_or_else(|| format_err!("Expected function"))?;
    let arg = arg
        .any_ref()
        .flatten()
        .ok_or_else(|| format_err!("Expected arg"))?;

    let arg = arg.to_owned_rooted(caller)?;
    Ok(DartCallback { callback, arg })
}

fn duration_from_args(arg: &Val) -> Result<Duration> {
    let micros = arg.i64().ok_or_else(|| format_err!("Expected i64"))?;
    Ok(Duration::from_micros(micros as u64))
}

/// `scheduleOnce`, implemented with `func_new` since the callback parameter is a typed funcref.
pub fn func_schedule_once<E: DartEmbedder>(
    mut caller: Caller<'_, E>,
    args: &[Val],
    results: &mut [Val],
) -> Result<()> {
    let delay = duration_from_args(&args[0])?;
    let callback = callback_from_args(&mut caller, &args[1], &args[2])?;

    let result = caller.data_mut().schedule_once(delay, callback)?;
    results[0] = Val::ExternRef(Some(ExternRef::new(caller, result)?));
    Ok(())
}

/// `scheduleRepeated`, implemented with `func_new` since the callback parameter is a typed funcref.
pub fn func_schedule_repeated<E: DartEmbedder>(
    mut caller: Caller<'_, E>,
    args: &[Val],
    results: &mut [Val],
) -> Result<()> {
    let interval = duration_from_args(&args[0])?;
    let callback = callback_from_args(&mut caller, &args[1], &args[2])?;

    let result = caller.data_mut().schedule_repeated(interval, callback)?;
    results[0] = Val::ExternRef(Some(ExternRef::new(caller, result)?));
    Ok(())
}

/// `queueMicrotask`, implemented with `func_new` since the callback parameter is a typed funcref.
pub fn func_queue_microtask<E: DartEmbedder>(
    mut caller: Caller<'_, E>,
    args: &[Val],
    _results: &mut [Val],
) -> Result<()> {
    let callback = callback_from_args(&mut caller, &args[0], &args[1])?;
    caller.data_mut().queue_microtask(callback)
}

pub fn func_clear_schedule<E: DartEmbedder>(
    mut caller: Caller<'_, E>,
    schedule: Rooted<ExternRef>,
) -> Result<()> {
    let schedule: &mut E::Timer = externref_mut(&mut caller, &schedule)?;
    schedule.clear_schedule();

    Ok(())
}

#[derive(Clone, Debug)]
pub struct DartCallback {
    callback: Func,
    arg: OwnedRooted<AnyRef>,
}

impl DartCallback {
    pub fn invoke(&self, mut store: impl AsContextMut) -> Result<()> {
        let arg = self.arg.to_rooted(&mut store);

        let param = Val::AnyRef(Some(arg));
        self.callback.call(&mut store, &[param], &mut [])
    }
}

/// Handle to a scheduled Dart timer.
pub trait DartSchedule: 'static + Any + Send + Sync {
    /// Called from Dart when `Timer.cancel` is called.
    fn clear_schedule(&mut self);
}
