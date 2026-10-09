use alloc::{borrow::ToOwned, boxed::Box};
use wasmtime::{Caller, ExternRef, Result, Rooted};

use crate::{
    DartEmbedder,
    string::DartString,
    utils::{externref_ref, null_check},
};

pub fn func_f64_to_string<'a, E: DartEmbedder>(
    caller: Caller<'a, E>,
    value: f64,
) -> Result<Rooted<ExternRef>> {
    let mut buffer = ryu_js::Buffer::new();
    let str = buffer.format(value);

    let str: Box<str> = if (value % 1.0) == 0.0 && !str.contains("e") {
        let mut str = str.to_owned();
        str.push_str(".0");

        str.into_boxed_str()
    } else {
        Box::from(str)
    };

    DartString::new_externref(caller, str)
}

pub fn func_f64_to_fixed<'a, E: DartEmbedder>(
    caller: Caller<'a, E>,
    value: f64,
    fraction_digits: i32,
) -> Result<Rooted<ExternRef>> {
    let mut buffer = ryu_js::Buffer::new();
    let str = buffer.format_to_fixed(value, fraction_digits as u8);

    DartString::new_externref(caller, str)
}

pub fn func_double_try_parse<'a, E: DartEmbedder>(
    mut caller: Caller<'a, E>,
    source: Option<Rooted<ExternRef>>,
) -> Result<Option<Rooted<ExternRef>>> {
    let source = null_check(source)?;
    let dart = DartString::from_externref(&mut caller, &source)?;
    let Ok(parsed) = dart.as_ref().parse::<f64>() else {
        return Ok(None);
    };

    Ok(Some(ExternRef::new(caller, parsed)?))
}

pub fn func_parse_result_get_double<'a, E: DartEmbedder>(
    caller: Caller<'a, E>,
    source: Option<Rooted<ExternRef>>,
) -> Result<f64> {
    let source = null_check(source)?;
    let value: &f64 = externref_ref(&caller, &source)?;

    Ok(*value)
}

pub fn func_double_parse_infallible<'a, E: DartEmbedder>(
    mut caller: Caller<'a, E>,
    source: Option<Rooted<ExternRef>>,
) -> Result<f64> {
    let source = null_check(source)?;
    let dart = DartString::from_externref(&mut caller, &source)?;
    let parsed: f64 = dart.as_ref().parse()?;

    Ok(parsed)
}
