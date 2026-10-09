use alloc::{borrow::ToOwned, boxed::Box, format, string::String};
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

// The following methods must behave like `Number.prototype.toExponential` and `toPrecision` in
// JavaScript. For simplicity, we use Rust's float formatting. It is correctly rounded, but resolves
// ties (where the exact binary value is halfway between two candidates) to even instead of rounding
// up like JavaScript. For instance, `2.5.toStringAsExponential(0)` is `2e+0` instead of `3e+0`.

/// Like JavaScript, we format `-0.0` without a sign (the Dart SDK adds it back where needed).
fn without_negative_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

/// Turns an exponent from Rust's float formatting (`1.5e3`) into JavaScript's style (`1.5e+3`).
fn js_exponent_sign(mut formatted: String) -> String {
    let index = formatted.find('e').unwrap() + 1;
    if formatted.as_bytes().get(index) != Some(&b'-') {
        formatted.insert(index, '+');
    }
    formatted
}

pub fn func_f64_to_exponential<'a, E: DartEmbedder>(
    caller: Caller<'a, E>,
    value: f64,
) -> Result<Rooted<ExternRef>> {
    let value = without_negative_zero(value);
    DartString::new_externref(caller, js_exponent_sign(format!("{value:e}")))
}

pub fn func_f64_to_exponential_with_fraction_digits<'a, E: DartEmbedder>(
    caller: Caller<'a, E>,
    value: f64,
    fraction_digits: i32,
) -> Result<Rooted<ExternRef>> {
    let value = without_negative_zero(value);
    let fraction_digits = fraction_digits as usize;
    let formatted = format!("{value:.fraction_digits$e}");
    DartString::new_externref(caller, js_exponent_sign(formatted))
}

pub fn func_f64_to_precision<'a, E: DartEmbedder>(
    caller: Caller<'a, E>,
    value: f64,
    precision: i32,
) -> Result<Rooted<ExternRef>> {
    let precision = precision as usize;
    // Rounding to `precision` significant digits also gives us the exponent for the rounded value.
    let formatted = format!("{:.*e}", precision - 1, value.abs());
    let (mantissa, exponent) = formatted.split_once('e').unwrap();
    let exponent: i32 = exponent.parse()?;

    let mut result = String::with_capacity(precision + 8);
    if value < 0.0 {
        result.push('-');
    }

    if exponent < -6 || exponent >= precision as i32 {
        result.push_str(mantissa);
        result.push('e');
        if exponent >= 0 {
            result.push('+');
        }
        result.push_str(&format!("{exponent}"));
    } else {
        let digits = mantissa.chars().filter(|c| *c != '.');
        if exponent >= 0 {
            // At most `precision` integer digits, followed by remaining digits as fractional part.
            for (i, digit) in digits.enumerate() {
                if i == exponent as usize + 1 {
                    result.push('.');
                }
                result.push(digit);
            }
        } else {
            result.push_str("0.");
            for _ in 0..(-exponent - 1) {
                result.push('0');
            }
            result.extend(digits);
        }
    }

    DartString::new_externref(caller, result)
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
