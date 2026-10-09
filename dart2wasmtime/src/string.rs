use core::mem;

use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
use wasmtime::{
    AsContextMut, Caller, ExternRef, Result, Rooted, StoreContext, Val, bail, format_err,
};

use crate::{
    DartEmbedder,
    utils::{externref_mut, externref_ref, null_check, null_check_ref},
};

#[derive(Clone, Default, PartialEq, PartialOrd, Eq, Ord)]
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

    pub fn func_from_ascii_bytes<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        args: &[Val],
        results: &mut [Val],
    ) -> Result<()> {
        let char_codes = null_check_ref(args[0].unwrap_anyref())?.unwrap_array(&caller)?;
        let start = args[1].unwrap_i32();
        let length = args[2].unwrap_i32();

        let bytes = if start == 0 && length as u32 == char_codes.len(&caller)? {
            let mut bytes = vec![0; length as usize].into_boxed_slice();
            char_codes.copy_to_i8_slice(&mut caller, &mut bytes)?;
            bytes
        } else {
            let mut bytes = Box::new_uninit_slice(length as usize);
            for i in 0..length {
                let value = char_codes.get(&mut caller, (start + i) as u32)?;
                bytes[i as usize].write(value.unwrap_i32() as u8);
            }

            unsafe { bytes.assume_init() }
        };

        let contents: Box<str> = unsafe {
            // SAFETY: All bytes have been initialized with ASCII characters above,
            // which makes them valid UTF-8.
            alloc::str::from_boxed_utf8_unchecked(bytes)
        };

        results[0] = Val::ExternRef(Some(ExternRef::new(
            &mut caller,
            DartString::from(contents),
        )?));
        Ok(())
    }

    pub fn func_from_char_code_array<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        args: &[Val],
        results: &mut [Val],
    ) -> Result<()> {
        let char_codes = null_check_ref(args[0].unwrap_anyref())?.unwrap_array(&caller)?;
        let start = args[1].unwrap_i32() as u32;
        let length = args[2].unwrap_i32() as u32;

        let mut elements = Vec::with_capacity(usize::try_from(length)?);
        for i in 0..length {
            let code_unit = char_codes.get(&mut caller, start + i)?;
            elements.push(code_unit.unwrap_i32() as u16);
        }
        let string = String::from_utf16(&elements)?;

        results[0] = Val::ExternRef(Some(ExternRef::new(
            &mut caller,
            DartString::from(string.into_boxed_str()),
        )?));
        Ok(())
    }

    pub fn new_externref(
        caller: impl AsContextMut,
        contents: impl Into<Box<str>>,
    ) -> Result<Rooted<ExternRef>> {
        ExternRef::new(caller, DartString::from(contents.into()))
    }

    /// The length of this string in UTF-16 code units, which is how Dart measures strings.
    fn utf16_len(&self) -> usize {
        self.contents.chars().map(char::len_utf16).sum()
    }

    /// Converts a Dart (UTF-16 code unit) index into a byte offset into [Self::contents].
    pub(crate) fn byte_offset(&self, index: i32) -> Result<usize> {
        let index = usize::try_from(index)?;
        let mut utf16_offset = 0;
        for (byte_offset, char) in self.contents.char_indices() {
            if utf16_offset == index {
                return Ok(byte_offset);
            }

            utf16_offset += char.len_utf16();
            if utf16_offset > index {
                bail!("Index {index} splits a surrogate pair, which is not supported");
            }
        }

        if utf16_offset == index {
            Ok(self.contents.len())
        } else {
            bail!("Index {index} out of bounds")
        }
    }

    /// Converts a byte offset into [Self::contents] into a Dart (UTF-16 code unit) index.
    pub(crate) fn utf16_index(&self, byte_offset: usize) -> i32 {
        self.contents[..byte_offset]
            .chars()
            .map(char::len_utf16)
            .sum::<usize>() as i32
    }

    pub fn func_string_length<E: DartEmbedder>(
        caller: Caller<'_, E>,
        string: Option<Rooted<ExternRef>>,
    ) -> Result<i32> {
        let string: &Self = externref_ref(&caller, &null_check(string)?)?;
        Ok(string.utf16_len() as i32)
    }

    pub fn func_string_equals<E: DartEmbedder>(
        caller: Caller<'_, E>,
        a: Option<Rooted<ExternRef>>,
        b: Option<Rooted<ExternRef>>,
    ) -> Result<i32> {
        let a = Self::from_externref(&caller, &null_check(a)?)?;
        let b = Self::from_externref(&caller, &null_check(b)?)?;

        Ok(if a.eq(b) { 1 } else { 0 })
    }

    pub fn func_string_compare<E: DartEmbedder>(
        caller: Caller<'_, E>,
        a: Option<Rooted<ExternRef>>,
        b: Option<Rooted<ExternRef>>,
    ) -> Result<i32> {
        let a = Self::from_externref(&caller, &null_check(a)?)?;
        let b = Self::from_externref(&caller, &null_check(b)?)?;

        Ok(match a.cmp(b) {
            core::cmp::Ordering::Less => -1,
            core::cmp::Ordering::Equal => 0,
            core::cmp::Ordering::Greater => 1,
        })
    }

    pub fn func_string_code_unit_at<E: DartEmbedder>(
        caller: Caller<'_, E>,
        string: Option<Rooted<ExternRef>>,
        index: i32,
    ) -> Result<i32> {
        let string = Self::from_externref(&caller, &null_check(string)?)?;
        let Some(code_unit) = string.contents.encode_utf16().nth(usize::try_from(index)?) else {
            bail!("out of bounds")
        };

        Ok(code_unit as i32)
    }

    pub fn func_index_of_string<E: DartEmbedder>(
        caller: Caller<'_, E>,
        string: Option<Rooted<ExternRef>>,
        needle: Option<Rooted<ExternRef>>,
        start: i32,
    ) -> Result<i32> {
        let string = Self::from_externref(&caller, &null_check(string)?)?;
        let needle = Self::from_externref(&caller, &null_check(needle)?)?;

        let start = string.byte_offset(start)?;
        Ok(match string.contents[start..].find(needle.as_ref()) {
            Some(index) => string.utf16_index(start + index),
            None => -1,
        })
    }

    pub fn func_last_index_of_string<E: DartEmbedder>(
        caller: Caller<'_, E>,
        string: Option<Rooted<ExternRef>>,
        needle: Option<Rooted<ExternRef>>,
        start: i32,
    ) -> Result<i32> {
        let string = Self::from_externref(&caller, &null_check(string)?)?;
        let needle = Self::from_externref(&caller, &null_check(needle)?)?;

        // The Dart side clamps start to `length - needle.length`, which is negative if the needle
        // is longer than the string.
        if start < 0 {
            return Ok(-1);
        }

        // Only consider matches starting at or before `start`. A match starting at `start` ends
        // on a char boundary, so rounding down doesn't skip any valid match.
        let start = string.byte_offset(start)?;
        let mut end = (start + needle.contents.len()).min(string.contents.len());
        while !string.contents.is_char_boundary(end) {
            end -= 1;
        }

        Ok(match string.contents[..end].rfind(needle.as_ref()) {
            Some(index) => string.utf16_index(index),
            None => -1,
        })
    }

    pub fn func_replace_all_string<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        string: Option<Rooted<ExternRef>>,
        needle: Option<Rooted<ExternRef>>,
        replacement: Option<Rooted<ExternRef>>,
    ) -> Result<Option<Rooted<ExternRef>>> {
        let string = Self::from_externref(&caller, &null_check(string)?)?;
        let needle = Self::from_externref(&caller, &null_check(needle)?)?;
        let replacement = Self::from_externref(&caller, &null_check(replacement)?)?;

        let replaced = string
            .as_ref()
            .replace(needle.as_ref(), replacement.as_ref());
        Ok(Some(Self::new_externref(&mut caller, replaced)?))
    }

    pub fn func_substring<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        string: Option<Rooted<ExternRef>>,
        start: i32,
        end: i32,
    ) -> Result<Option<Rooted<ExternRef>>> {
        let string = Self::from_externref(&caller, &null_check(string)?)?;
        let start = string.byte_offset(start)?;
        let end = string.byte_offset(end)?;
        let Some(substring) = string.contents.get(start..end) else {
            bail!("Invalid substring range")
        };

        let substring = Box::<str>::from(substring);
        Ok(Some(Self::new_externref(&mut caller, substring)?))
    }

    pub fn func_to_lower_case<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        string: Option<Rooted<ExternRef>>,
    ) -> Result<Option<Rooted<ExternRef>>> {
        let string = Self::from_externref(&caller, &null_check(string)?)?;
        let lower = string.contents.to_lowercase();
        Ok(Some(Self::new_externref(&mut caller, lower)?))
    }

    pub fn func_to_upper_case<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        string: Option<Rooted<ExternRef>>,
    ) -> Result<Option<Rooted<ExternRef>>> {
        let string = Self::from_externref(&caller, &null_check(string)?)?;
        let upper = string.contents.to_uppercase();
        Ok(Some(Self::new_externref(&mut caller, upper)?))
    }

    pub fn func_concat<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        a: Option<Rooted<ExternRef>>,
        b: Option<Rooted<ExternRef>>,
    ) -> Result<Option<Rooted<ExternRef>>> {
        let a = Self::from_externref(&caller, &null_check(a)?)?;
        let b = Self::from_externref(&caller, &null_check(b)?)?;

        let mut concat = String::with_capacity(a.contents.len() + b.contents.len());
        concat.push_str(a.as_ref());
        concat.push_str(b.as_ref());
        Ok(Some(Self::new_externref(&mut caller, concat)?))
    }

    pub fn func_replace_range<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        string: Option<Rooted<ExternRef>>,
        start: i32,
        end: i32,
        replacement: Option<Rooted<ExternRef>>,
    ) -> Result<Option<Rooted<ExternRef>>> {
        let string = Self::from_externref(&caller, &null_check(string)?)?;
        let replacement = Self::from_externref(&caller, &null_check(replacement)?)?;

        let start = string.byte_offset(start)?;
        let end = string.byte_offset(end)?;
        if start > end {
            bail!("Invalid range to replace")
        }

        let mut replaced = String::from(string.as_ref());
        replaced.replace_range(start..end, replacement.as_ref());
        Ok(Some(Self::new_externref(&mut caller, replaced)?))
    }

    pub fn func_to_code_units<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        args: &[Val],
        _results: &mut [Val],
    ) -> Result<()> {
        let string = null_check_ref(args[0].unwrap_externref())?;
        let out = null_check_ref(args[1].unwrap_anyref())?.unwrap_array(&caller)?;
        let start = args[2].unwrap_i32() as u32;

        // Copy the code units first since we can't write to the array while borrowing the string.
        let code_units: Vec<u16> = Self::from_externref(&caller, string)?
            .contents
            .encode_utf16()
            .collect();

        for (i, code_unit) in code_units.into_iter().enumerate() {
            out.set(&mut caller, start + i as u32, Val::I32(code_unit as i32))?;
        }
        Ok(())
    }

    pub fn func_json_encode_string<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        string: Option<Rooted<ExternRef>>,
    ) -> Result<Rooted<ExternRef>> {
        let string: &Self = externref_ref(&caller, &null_check(string)?)?;
        let as_json: String = match serde_json::to_string(string.as_ref()) {
            Ok(json) => json,
            Err(_) => bail!("Could not format json"),
        };

        Self::new_externref(&mut caller, as_json)
    }

    pub fn func_repeat<E: DartEmbedder>(
        mut caller: Caller<'_, E>,
        string: Option<Rooted<ExternRef>>,
        amount: i32,
    ) -> Result<Option<Rooted<ExternRef>>> {
        let string = null_check(string)?;
        let string = Self::from_externref(&caller, &string)?.as_ref();
        let repeated = string.repeat(usize::try_from(amount)?);

        Ok(Some(Self::new_externref(&mut caller, repeated)?))
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
