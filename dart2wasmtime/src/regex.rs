use core::ops::Range;

use alloc::{boxed::Box, string::String};
use regress::{Match, Regex};
use wasmtime::{Caller, ExternRef, OwnedRooted, Result, Rooted, StoreContext, bail};

use crate::{
    DartEmbedder,
    string::DartString,
    utils::{externref_ref, null_check, null_check_ref},
};

struct DartRegex {
    regex: Regex,
}

impl DartRegex {
    fn from_externref<'a, E: DartEmbedder>(
        ctx: impl Into<StoreContext<'a, E>>,
        r: &Rooted<ExternRef>,
    ) -> Result<&'a Self> {
        externref_ref(ctx, r)
    }
}

struct DartMatch {
    search_string: OwnedRooted<ExternRef>,
    regex_match: Match,
}

impl DartMatch {
    fn from_externref<'a, E: DartEmbedder>(
        ctx: impl Into<StoreContext<'a, E>>,
        r: &Rooted<ExternRef>,
    ) -> Result<&'a Self> {
        externref_ref(ctx, r)
    }

    fn search_string<'a, E: DartEmbedder>(
        &self,
        ctx: impl Into<StoreContext<'a, E>>,
    ) -> Result<&'a DartString> {
        let Some(data) = self.search_string.data(ctx)? else {
            bail!("Invalid externref")
        };
        let Some(string) = data.downcast_ref() else {
            bail!("Invalid externref")
        };

        Ok(string)
    }

    fn named_group(&self, index: i32) -> Result<(&str, Option<Range<usize>>)> {
        let Some(group) = self.regex_match.named_groups().nth(usize::try_from(index)?) else {
            bail!("invalid named group index")
        };

        Ok(group)
    }
}

pub fn func_string_replace_all_regexp<E: DartEmbedder>(
    caller: Caller<'_, E>,
    string: Option<Rooted<ExternRef>>,
    needle: Option<Rooted<ExternRef>>,
    replacement: Option<Rooted<ExternRef>>,
) -> Result<Option<Rooted<ExternRef>>> {
    let string = null_check(string)?;
    let haystack = DartString::from_externref(&caller, &string)?;
    let needle = DartRegex::from_externref(&caller, null_check_ref(needle.as_ref())?)?;
    let replacement = DartString::from_externref(&caller, null_check_ref(replacement.as_ref())?)?;
    let haystack = haystack.as_ref();

    let mut result = None::<(String, usize)>;

    for m in needle.regex.find_iter(haystack) {
        let (buffer, last_end) =
            result.get_or_insert_with(|| (String::with_capacity(haystack.len()), 0));

        buffer.push_str(&haystack[*last_end..m.start()]);
        buffer.push_str(replacement.as_ref());
        *last_end = m.end();
    }

    Ok(match result {
        // Nothing changed? Return original reference.
        None => Some(string),
        Some((mut replacement, last_end)) => {
            replacement.push_str(&haystack[last_end..]);
            Some(DartString::new_externref(caller, replacement)?)
        }
    })
}

pub fn func_regexp_create_or_fail_with_string<E: DartEmbedder>(
    caller: Caller<'_, E>,
    string: Option<Rooted<ExternRef>>,
    multiline: i32,
    case_sensitive: i32,
    unicode: i32,
    dot_all: i32,
) -> Result<Rooted<ExternRef>> {
    let string = DartString::from_externref(&caller, &null_check(string)?)?;
    let mut flags = regress::Flags::default();
    flags.multiline = multiline != 0;
    flags.icase = case_sensitive == 0;
    flags.unicode = unicode != 0;
    flags.dot_all = dot_all != 0;

    Ok(match Regex::with_flags(string.as_ref(), flags) {
        Ok(regex) => ExternRef::new(caller, DartRegex { regex })?,
        Err(err) => DartString::new_externref(caller, err.text)?,
    })
}

pub fn func_regexp_is_regexp<E: DartEmbedder>(
    caller: Caller<'_, E>,
    regexp: Rooted<ExternRef>,
) -> Result<i32> {
    let Some(data) = regexp.data(&caller)? else {
        bail!("Invalid externref")
    };

    Ok(if data.is::<DartRegex>() { 1 } else { 0 })
}

pub fn func_regexp_escape<E: DartEmbedder>(
    caller: Caller<'_, E>,
    string: Option<Rooted<ExternRef>>,
) -> Result<Rooted<ExternRef>> {
    let string = DartString::from_externref(&caller, &null_check(string)?)?;
    let escaped = regress::escape(string.as_ref());

    DartString::new_externref(caller, escaped)
}

pub fn func_regexp_match<E: DartEmbedder>(
    mut caller: Caller<'_, E>,
    regexp: Option<Rooted<ExternRef>>,
    string: Option<Rooted<ExternRef>>,
    start: i32,
    as_prefix: i32,
) -> Result<Option<Rooted<ExternRef>>> {
    let string = null_check(string)?;
    let regexp = DartRegex::from_externref(&caller, null_check_ref(regexp.as_ref())?)?;
    let haystack = DartString::from_externref(&caller, &string)?;

    let start = haystack.byte_offset(start)?;
    let Some(regex_match) = regexp.regex.find_from(haystack.as_ref(), start).next() else {
        return Ok(None);
    };

    // Matches are found leftmost-first, so if there is a match starting at `start`, it's the
    // first one returned.
    if as_prefix != 0 && regex_match.start() != start {
        return Ok(None);
    }

    let search_string = string.to_owned_rooted(&mut caller)?;
    Ok(Some(ExternRef::new(
        caller,
        DartMatch {
            search_string,
            regex_match,
        },
    )?))
}

pub fn func_regexp_match_get_start<E: DartEmbedder>(
    caller: Caller<'_, E>,
    rmatch: Option<Rooted<ExternRef>>,
) -> Result<i32> {
    let rmatch = DartMatch::from_externref(&caller, &null_check(rmatch)?)?;
    let search_str = rmatch.search_string(&caller)?;
    Ok(search_str.utf16_index(rmatch.regex_match.start()))
}

pub fn func_regexp_match_get_end<E: DartEmbedder>(
    caller: Caller<'_, E>,
    rmatch: Option<Rooted<ExternRef>>,
) -> Result<i32> {
    let rmatch = DartMatch::from_externref(&caller, &null_check(rmatch)?)?;
    let search_str = rmatch.search_string(&caller)?;
    Ok(search_str.utf16_index(rmatch.regex_match.end()))
}

pub fn func_regexp_match_group_count<E: DartEmbedder>(
    caller: Caller<'_, E>,
    rmatch: Option<Rooted<ExternRef>>,
) -> Result<i32> {
    let rmatch = DartMatch::from_externref(&caller, &null_check(rmatch)?)?;
    Ok(i32::try_from(rmatch.regex_match.captures.len())?)
}

pub fn func_regexp_match_group<E: DartEmbedder>(
    caller: Caller<'_, E>,
    rmatch: Option<Rooted<ExternRef>>,
    index: i32,
) -> Result<Option<Rooted<ExternRef>>> {
    let rmatch = DartMatch::from_externref(&caller, &null_check(rmatch)?)?;
    let index = usize::try_from(index)?;
    if index > rmatch.regex_match.captures.len() {
        bail!("invalid group index")
    }

    // Groups that didn't participate in the match are null.
    let Some(range) = rmatch.regex_match.group(index) else {
        return Ok(None);
    };

    let group: Box<str> = rmatch.search_string(&caller)?.as_ref()[range].into();
    Ok(Some(DartString::new_externref(caller, group)?))
}

pub fn func_regexp_match_get_named_groups<E: DartEmbedder>(
    caller: Caller<'_, E>,
    rmatch: Option<Rooted<ExternRef>>,
) -> Result<i32> {
    let rmatch = DartMatch::from_externref(&caller, &null_check(rmatch)?)?;
    Ok(i32::try_from(rmatch.regex_match.named_groups().count())?)
}

pub fn func_regexp_match_get_group_name<E: DartEmbedder>(
    caller: Caller<'_, E>,
    rmatch: Option<Rooted<ExternRef>>,
    index: i32,
) -> Result<Rooted<ExternRef>> {
    let rmatch = DartMatch::from_externref(&caller, &null_check(rmatch)?)?;
    let (name, _) = rmatch.named_group(index)?;
    let name: Box<str> = name.into();

    DartString::new_externref(caller, name)
}

pub fn func_regexp_match_get_group_by_name<E: DartEmbedder>(
    caller: Caller<'_, E>,
    rmatch: Option<Rooted<ExternRef>>,
    name_index: i32,
) -> Result<Option<Rooted<ExternRef>>> {
    let rmatch = null_check(rmatch)?;
    let rmatch = DartMatch::from_externref(&caller, &rmatch)?;
    let (_, range) = rmatch.named_group(name_index)?;
    let Some(range) = range else {
        return Ok(None);
    };

    let group: Box<str> = rmatch.search_string(&caller)?.as_ref()[range].into();
    Ok(Some(DartString::new_externref(caller, group)?))
}
