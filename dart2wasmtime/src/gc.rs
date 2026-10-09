use wasmtime::{AnyRef, Caller, ExternRef, Result, Rooted};

use crate::{DartEmbedder, utils::null_check};

// wasmtime doesn't support weak references, so we use strong references instead (which is still
// allowed by Dart).

pub fn func_weak_ref_create<E: DartEmbedder>(
    caller: Caller<'_, E>,
    original_value: Option<Rooted<AnyRef>>,
) -> Result<Rooted<ExternRef>> {
    ExternRef::convert_any(caller, null_check(original_value)?)
}

pub fn func_weak_ref_get<E: DartEmbedder>(
    caller: Caller<'_, E>,
    weak: Option<Rooted<ExternRef>>,
) -> Result<Option<Rooted<AnyRef>>> {
    let weak = null_check(weak)?;
    Ok(Some(AnyRef::convert_extern(caller, weak)?))
}
