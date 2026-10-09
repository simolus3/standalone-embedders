use core::{any::Any, time::Duration};

use wasmtime::{AnyRef, AsContextMut, Caller, ExternRef, Func, OwnedRooted, Result, Rooted, Val};

use crate::{
    DartEmbedder,
    utils::{externref_mut, null_check},
};

pub fn func_schedule_once<E: DartEmbedder>(
    mut caller: Caller<'_, E>,
    delay: i64,
    callback: Func,
    arg: Option<Rooted<AnyRef>>,
) -> Result<Rooted<ExternRef>> {
    let arg = match arg {
        None => None,
        Some(arg) => Some(arg.to_owned_rooted(&mut caller)?),
    };
    let callback = DartCallback { callback, arg };
    let delay = Duration::from_micros(delay as u64);

    let result = caller.data_mut().schedule_once(delay, callback)?;
    ExternRef::new(caller, result)
}

pub fn func_schedule_repeated<E: DartEmbedder>(
    mut caller: Caller<'_, E>,
    interval: i64,
    callback: Func,
    arg: Option<Rooted<AnyRef>>,
) -> Result<Rooted<ExternRef>> {
    let arg = match arg {
        None => None,
        Some(arg) => Some(arg.to_owned_rooted(&mut caller)?),
    };
    let callback = DartCallback { callback, arg };
    let interval = Duration::from_micros(interval as u64);

    let result = caller.data_mut().schedule_repeated(interval, callback)?;
    ExternRef::new(caller, result)
}

pub fn func_queue_microtask<E: DartEmbedder>(
    mut caller: Caller<'_, E>,
    callback: Func,
    arg: Option<Rooted<AnyRef>>,
) -> Result<()> {
    let arg = match arg {
        None => None,
        Some(arg) => Some(arg.to_owned_rooted(&mut caller)?),
    };
    let callback = DartCallback { callback, arg };
    caller.data_mut().queue_microtask(callback)
}

pub fn func_clear_schedule<E: DartEmbedder>(
    mut caller: Caller<'_, E>,
    schedule: Option<Rooted<ExternRef>>,
) -> Result<()> {
    let schedule = null_check(schedule)?;
    let schedule: &mut E::Timer = externref_mut(&mut caller, &schedule)?;
    schedule.clear_schedule();

    Ok(())
}

#[derive(Clone, Debug)]
pub struct DartCallback {
    callback: Func,
    arg: Option<OwnedRooted<AnyRef>>,
}

impl DartCallback {
    pub fn invoke(&self, mut store: impl AsContextMut) -> Result<()> {
        let arg = self.arg.as_ref().map(|arg| arg.to_rooted(&mut store));

        let param = Val::AnyRef(arg);
        self.callback.call(&mut store, &[param], &mut [])
    }
}

pub trait DartSchedule: 'static + Any + Send + Sync {
    fn clear_schedule(&mut self);
}
