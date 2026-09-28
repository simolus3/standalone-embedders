use wasmtime::{Caller, ExternRef, OwnedRooted, Result, Rooted};

use crate::{
    DartEmbedder,
    string::DartString,
    utils::{externref_ref, null_check},
};

pub struct StackTrace {
    formatted: OwnedRooted<ExternRef>,
}

impl StackTrace {
    pub fn func_current<E: DartEmbedder>(mut caller: Caller<'_, E>) -> Result<Rooted<ExternRef>> {
        let str = ExternRef::new(&mut caller, DartString::default())?;
        let owned = str.to_owned_rooted(&mut caller)?;

        ExternRef::new(caller, Self { formatted: owned })
    }

    pub fn func_to_string<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        trace: Option<Rooted<ExternRef>>,
    ) -> Result<Rooted<ExternRef>> {
        let trace: &Self = externref_ref(&caller, &null_check(trace)?)?;
        Ok(trace.formatted.clone().to_rooted(&mut caller))
    }
}
