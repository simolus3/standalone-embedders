use core::any::Any;

use wasmtime::{ExternRef, GcRef, Result, Rooted, StoreContext, StoreContextMut, bail, format_err};

use crate::DartEmbedder;

pub fn null_check<T: GcRef>(r: Option<Rooted<T>>) -> Result<Rooted<T>> {
    r.ok_or_else(|| format_err!("Unexpected null"))
}

pub fn null_check_ref<T: GcRef>(r: Option<&Rooted<T>>) -> Result<&Rooted<T>> {
    r.ok_or_else(|| format_err!("Unexpected null"))
}

pub fn externref_ref<'a, E: DartEmbedder, T: Any>(
    ctx: impl Into<StoreContext<'a, E>>,
    r: &Rooted<ExternRef>,
) -> Result<&'a T> {
    let Some(data) = r.data(ctx)? else {
        bail!("Invalid externref")
    };

    let Some(data) = data.downcast_ref() else {
        bail!("Invalid externref")
    };

    Ok(data)
}

pub fn externref_mut<'a, E: DartEmbedder, T: Any>(
    ctx: impl Into<StoreContextMut<'a, E>>,
    r: &'a Rooted<ExternRef>,
) -> Result<&'a mut T> {
    let Some(data) = r.data_mut(ctx)? else {
        bail!("Invalid externref")
    };

    let Some(data) = data.downcast_mut() else {
        bail!("Invalid externref")
    };

    Ok(data)
}
