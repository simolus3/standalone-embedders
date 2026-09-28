use core::mem;

use alloc::{boxed::Box, format, string::String};
use wasmtime::{Caller, ExternRef, Result, Rooted, StoreContext, bail, format_err};

use crate::{
    DartEmbedder,
    utils::{externref_mut, externref_ref, null_check},
};

#[derive(Clone, Default)]
#[repr(transparent)]
pub struct DartString {
    contents: Box<str>,
}

impl DartString {
    pub fn from_externref<'a, E: DartEmbedder>(
        ctx: impl Into<StoreContext<'a, E>>,
        r: &Rooted<ExternRef>,
    ) -> Result<&'a Self> {
        externref_ref(ctx, r)
    }

    pub fn func_string_length<E: DartEmbedder>(
        caller: Caller<'_, E>,
        string: Option<Rooted<ExternRef>>,
    ) -> Result<i32> {
        let string: &Self = externref_ref(&caller, &null_check(string)?)?;
        Ok(string.contents.len() as i32)
    }

    /// Formats `value` like Dart's `int.toRadixString(radix)`: lowercase digits, with a leading
    /// `-` for negative values.
    pub fn from_i64(value: i64, radix: i32) -> Result<Self> {
        let sign = if value < 0 { "-" } else { "" };
        // unsigned_abs avoids overflowing on i64::MIN.
        let magnitude = value.unsigned_abs();

        let string = match radix {
            2 => format!("{sign}{magnitude:b}"),
            8 => format!("{sign}{magnitude:o}"),
            10 => format!("{value}"),
            16 => format!("{sign}{magnitude:x}"),
            3..=36 => {
                const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
                let radix = radix as u64;

                // Enough for u64::MAX in base 2, plus the sign.
                let mut buf = [0u8; 65];
                let mut pos = buf.len();
                let mut remaining = magnitude;
                loop {
                    pos -= 1;
                    buf[pos] = DIGITS[(remaining % radix) as usize];
                    remaining /= radix;
                    if remaining == 0 {
                        break;
                    }
                }
                if value < 0 {
                    pos -= 1;
                    buf[pos] = b'-';
                }

                // SAFETY: buf[pos..] only contains ASCII digits and '-'.
                String::from(unsafe { core::str::from_utf8_unchecked(&buf[pos..]) })
            }
            _ => bail!("Invalid radix: {radix}"),
        };

        Ok(string.into_boxed_str().into())
    }
}

impl AsRef<str> for DartString {
    fn as_ref(&self) -> &str {
        &self.contents
    }
}

impl From<Box<str>> for DartString {
    fn from(value: Box<str>) -> Self {
        Self { contents: value }
    }
}

#[derive(Default)]
pub struct StringBuffer {
    pub data: String,
}

impl StringBuffer {
    pub fn func_new<E: DartEmbedder>(caller: Caller<'_, E>) -> Result<Rooted<ExternRef>> {
        ExternRef::new(caller, Self::default())
    }

    pub fn func_write_string<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        buffer: Option<Rooted<ExternRef>>,
        string: Option<Rooted<ExternRef>>,
    ) -> Result<()> {
        let buffer = null_check(buffer)?;
        let string = null_check(string)?;

        // TODO: Can we avoid the clone here?
        let string: &DartString = externref_ref(&caller, &string)?;
        let string = string.clone();
        let buffer: &mut Self = externref_mut(&mut caller, &buffer)?;

        buffer.data.push_str(string.as_ref());
        Ok(())
    }

    pub fn func_write_char_code<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        buffer: Option<Rooted<ExternRef>>,
        code: i32,
    ) -> Result<()> {
        let buffer = null_check(buffer)?;

        let buffer: &mut Self = externref_mut(&mut caller, &buffer)?;

        buffer.data.push(
            char::from_u32(code as u32).ok_or_else(|| format_err!("Invalid char code {code}"))?,
        );
        Ok(())
    }

    pub fn func_clear<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        buffer: Option<Rooted<ExternRef>>,
    ) -> Result<()> {
        let buffer = null_check(buffer)?;
        let buffer: &mut Self = externref_mut(&mut caller, &buffer)?;

        buffer.data.clear();
        Ok(())
    }

    pub fn func_length<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        buffer: Option<Rooted<ExternRef>>,
    ) -> Result<i32> {
        let buffer = null_check(buffer)?;
        let buffer: &mut Self = externref_mut(&mut caller, &buffer)?;

        Ok(buffer.data.len() as i32)
    }

    pub fn func_to_string<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        buffer: Option<Rooted<ExternRef>>,
    ) -> Result<Rooted<ExternRef>> {
        let buffer = null_check(buffer)?;
        let buffer: Self = mem::take(externref_mut(&mut caller, &buffer)?);

        ExternRef::new(&mut caller, DartString::from(buffer.data.into_boxed_str()))
    }
}
