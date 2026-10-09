use wasmtime::{AnyRef, Caller, ExternRef, Result, Rooted};

use crate::DartEmbedder;

pub fn func_debugger<E: DartEmbedder>(
    mut caller: Caller<'_, E>,
    _message: Option<Rooted<ExternRef>>,
) -> Result<()> {
    caller.data_mut().debugger()
}

pub fn func_inspect<E: DartEmbedder>(
    mut caller: Caller<'_, E>,
    obj: Option<Rooted<AnyRef>>,
) -> Result<()> {
    caller.data_mut().inspect(obj)
}

pub fn func_timeline_stream_enabled() -> i32 {
    0
}

pub fn func_report_task_event(
    _task_id: i32,
    _flow_id: i32,
    _type: i32,
    _name: Option<Rooted<ExternRef>>,
    _arguments_as_json: Option<Rooted<ExternRef>>,
) -> i32 {
    0
}
