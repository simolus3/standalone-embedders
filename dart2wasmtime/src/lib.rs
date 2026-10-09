#![no_std]
extern crate alloc;

use core::time::Duration;

use alloc::string::String;
use wasmtime::{
    AnyRef, ArrayRef, ArrayRefPre, AsContextMut, Caller, ExternRef, ExternType, HeapType, Instance,
    Linker, Module, Result, Rooted, Val, ValType, bail, format_err,
};

use crate::{
    stack_trace::StackTrace,
    string::{DartString, StringBuffer},
    utils::null_check,
};

pub use event_loop::{DartCallback, DartSchedule};
pub use stopwatch::StopwatchFrequency;

mod developer;
mod event_loop;
mod numbers;
#[cfg(feature = "regex")]
mod regex;
mod stack_trace;
mod stopwatch;
mod string;
pub mod utils;

pub trait DartEmbedder: 'static {
    type Timer: DartSchedule;

    fn print(&self, _msg: &str) -> Result<()> {
        bail!("print not implemented")
    }

    fn random_int(&mut self, _secure: bool) -> Result<i64> {
        bail!("Randomness not implemented")
    }

    fn unix_timestamp(&mut self) -> Result<Duration> {
        bail!("No time source")
    }

    fn schedule_once(
        &mut self,
        _duration: Duration,
        _callback: DartCallback,
    ) -> Result<Self::Timer> {
        bail!("scheduleOnce not implemented")
    }

    fn schedule_repeated(
        &mut self,
        _duration: Duration,
        _callback: DartCallback,
    ) -> Result<Self::Timer> {
        bail!("scheduleOnce not implemented")
    }

    fn queue_microtask(&mut self, _callback: DartCallback) -> Result<()> {
        bail!("queueMicrotask not implemented")
    }

    fn monotonic_ticks(&mut self) -> Result<i64> {
        bail!("monotonic timer not implemented")
    }

    fn debugger(&mut self) -> Result<()> {
        Ok(())
    }

    fn inspect(&self, _obj: Option<Rooted<AnyRef>>) -> Result<()> {
        Ok(())
    }

    /// Whether we're running on Windows.
    ///
    /// Dart queries this to determine whether `Uri.toFilePath` should use forward or backward
    /// slashes.
    fn is_windows(&self) -> bool {
        if cfg!(windows) { true } else { false }
    }

    /// The base URI.
    ///
    /// This typically is the path of the running Dart entrypoint.
    fn base_uri(&self) -> Result<String> {
        bail!("base_uri not supported")
    }

    /// The frequency in which [Self::monotonic_ticks] are incremented.
    const MONOTONIC_FREQUENCY: StopwatchFrequency = StopwatchFrequency::MegaHertz;
}

pub fn add_dart_imports<E: DartEmbedder>(linker: &mut Linker<E>, module: &Module) -> Result<()> {
    for import in module.imports() {
        if import.module() != "dart" {
            continue;
        };

        let ExternType::Func(fn_ty) = import.ty() else {
            continue;
        };

        match import.name() {
            "scheduleOnce" => {
                linker.func_wrap("dart", import.name(), event_loop::func_schedule_once)?;
            }
            "scheduleRepeated" => {
                linker.func_wrap("dart", import.name(), event_loop::func_schedule_repeated)?;
            }
            "queueMicrotask" => {
                linker.func_wrap("dart", import.name(), event_loop::func_queue_microtask)?;
            }
            "clearSchedule" => {
                linker.func_wrap("dart", import.name(), event_loop::func_clear_schedule)?;
            }
            "currentTime" => {
                linker.func_wrap(
                    "dart",
                    import.name(),
                    |mut caller: Caller<'_, E>| -> Result<i64> {
                        caller
                            .data_mut()
                            .unix_timestamp()
                            .map(|ts| ts.as_micros() as i64)
                    },
                )?;
            }
            "stringFromAsciiBytes" => {
                linker.func_new(
                    "dart",
                    import.name(),
                    fn_ty,
                    DartString::func_from_ascii_bytes,
                )?;
            }
            "stringFromCharCodeArray" => {
                linker.func_new(
                    "dart",
                    import.name(),
                    fn_ty,
                    DartString::func_from_char_code_array,
                )?;
            }
            "stringLength" => {
                linker.func_wrap("dart", import.name(), DartString::func_string_length)?;
            }
            "stringEquals" => {
                linker.func_wrap("dart", import.name(), DartString::func_string_equals)?;
            }
            "stringCompare" => {
                linker.func_wrap("dart", import.name(), DartString::func_string_compare)?;
            }
            "stringCodeUnitAt" => {
                linker.func_wrap("dart", import.name(), DartString::func_string_code_unit_at)?;
            }
            "stringIndexOfString" => {
                linker.func_wrap("dart", import.name(), DartString::func_index_of_string)?;
            }
            "stringLastIndexOfString" => {
                linker.func_wrap("dart", import.name(), DartString::func_last_index_of_string)?;
            }
            "stringReplaceAllString" => {
                linker.func_wrap("dart", import.name(), DartString::func_replace_all_string)?;
            }
            "stringSubstring" => {
                linker.func_wrap("dart", import.name(), DartString::func_substring)?;
            }
            "stringToLowerCase" => {
                linker.func_wrap("dart", import.name(), DartString::func_to_lower_case)?;
            }
            "stringToUpperCase" => {
                linker.func_wrap("dart", import.name(), DartString::func_to_upper_case)?;
            }
            "stringConcat" => {
                linker.func_wrap("dart", import.name(), DartString::func_concat)?;
            }
            "stringRepeat" => {
                linker.func_wrap("dart", import.name(), DartString::func_repeat)?;
            }
            "stringReplaceRange" => {
                linker.func_wrap("dart", import.name(), DartString::func_replace_range)?;
            }
            "stringToCodeUnits" => {
                linker.func_new("dart", import.name(), fn_ty, DartString::func_to_code_units)?;
            }
            "monotonicClockFrequency" => {
                linker.func_wrap(
                    "dart",
                    import.name(),
                    stopwatch::func_monotonic_clock_frequency::<E>,
                )?;
            }
            "monotonicClockTicks" => {
                linker.func_wrap("dart", import.name(), stopwatch::func_monotonic_clock_ticks)?;
            }
            "doubleTryParse" => {
                linker.func_wrap("dart", import.name(), numbers::func_double_try_parse)?;
            }
            "tryParseResultGetDouble" => {
                linker.func_wrap("dart", import.name(), numbers::func_parse_result_get_double)?;
            }
            "doubleParseInfallible" => {
                linker.func_wrap("dart", import.name(), numbers::func_double_parse_infallible)?;
            }
            "i64ToString" => {
                linker.func_wrap(
                    "dart",
                    import.name(),
                    |mut caller: Caller<'_, E>,
                     value: i64,
                     radix: i32|
                     -> Result<Rooted<ExternRef>> {
                        ExternRef::new(&mut caller, DartString::from_i64(value, radix)?)
                    },
                )?;
            }
            "f64ToFixed" => {
                linker.func_wrap("dart", import.name(), numbers::func_f64_to_fixed)?;
            }
            "f64ToString" => {
                linker.func_wrap("dart", import.name(), numbers::func_f64_to_string)?;
            }
            "stringBufferCreate" => {
                linker.func_wrap("dart", import.name(), StringBuffer::func_new)?;
            }
            "stringBufferWriteString" => {
                linker.func_wrap("dart", import.name(), StringBuffer::func_write_string)?;
            }
            "stringBufferWriteCharCode" => {
                linker.func_wrap("dart", import.name(), StringBuffer::func_write_char_code)?;
            }
            "stringBufferClear" => {
                linker.func_wrap("dart", import.name(), StringBuffer::func_clear)?;
            }
            "stringBufferLength" => {
                linker.func_wrap("dart", import.name(), StringBuffer::func_length)?;
            }
            "stringBufferToString" => {
                linker.func_wrap("dart", import.name(), StringBuffer::func_to_string)?;
            }
            "print" => {
                linker.func_wrap(
                    "dart",
                    import.name(),
                    |caller: Caller<'_, E>, param: Option<Rooted<ExternRef>>| -> Result<()> {
                        let param = null_check(param)?;
                        let str = DartString::from_externref(&caller, &param)?;

                        let embedder = caller.data();
                        embedder.print(str.as_ref())
                    },
                )?;
            }
            "baseUri" => {
                linker.func_wrap(
                    "dart",
                    import.name(),
                    |caller: Caller<'_, E>| -> Result<Option<Rooted<ExternRef>>> {
                        let base_uri = caller.data().base_uri()?;
                        Ok(Some(DartString::new_externref(caller, base_uri)?))
                    },
                )?;
            }
            "isWindows" => {
                linker.func_wrap("dart", import.name(), |caller: Caller<'_, E>| -> i32 {
                    if caller.data().is_windows() { 1 } else { 0 }
                })?;
            }
            "stackTraceGetCurrent" => {
                linker.func_wrap("dart", import.name(), StackTrace::func_current)?;
            }
            "stackTraceToString" => {
                linker.func_wrap("dart", import.name(), StackTrace::func_to_string)?;
            }
            "mathPow" => {
                linker.func_wrap("dart", import.name(), libm::pow)?;
            }
            "mathAtan2" => {
                linker.func_wrap("dart", import.name(), libm::atan2)?;
            }
            "mathSin" => {
                linker.func_wrap("dart", import.name(), libm::sin)?;
            }
            "mathCos" => {
                linker.func_wrap("dart", import.name(), libm::cos)?;
            }
            "mathTan" => {
                linker.func_wrap("dart", import.name(), libm::tan)?;
            }
            "mathAsin" => {
                linker.func_wrap("dart", import.name(), libm::asin)?;
            }
            "mathAcos" => {
                linker.func_wrap("dart", import.name(), libm::acos)?;
            }
            "mathAtan" => {
                linker.func_wrap("dart", import.name(), libm::atan)?;
            }
            "mathExp" => {
                linker.func_wrap("dart", import.name(), libm::exp)?;
            }
            "mathLog" => {
                linker.func_wrap("dart", import.name(), libm::log)?;
            }
            "randomInt" => {
                linker.func_wrap(
                    "dart",
                    import.name(),
                    |mut caller: Caller<'_, E>| -> Result<i64> {
                        caller.data_mut().random_int(false)
                    },
                )?;
            }
            "randomIntSecure" => {
                linker.func_wrap(
                    "dart",
                    import.name(),
                    |mut caller: Caller<'_, E>| -> Result<i64> {
                        caller.data_mut().random_int(true)
                    },
                )?;
            }
            "jsonEncodeString" => {
                linker.func_wrap("dart", import.name(), DartString::func_json_encode_string)?;
            }
            "debugger" => {
                linker.func_wrap("dart", import.name(), developer::func_debugger)?;
            }
            "inspect" => {
                linker.func_wrap("dart", import.name(), developer::func_inspect)?;
            }
            "timelineStreamEnabled" => {
                linker.func_wrap(
                    "dart",
                    import.name(),
                    developer::func_timeline_stream_enabled,
                )?;
            }
            "reportTaskEvent" => {
                linker.func_wrap("dart", import.name(), developer::func_report_task_event)?;
            }
            #[cfg(feature = "regex")]
            "stringReplaceAllRegExp" => {
                linker.func_wrap("dart", import.name(), regex::func_string_replace_all_regexp)?;
            }
            #[cfg(feature = "regex")]
            "regexpCreateOrFailWithString" => {
                linker.func_wrap(
                    "dart",
                    import.name(),
                    regex::func_regexp_create_or_fail_with_string,
                )?;
            }
            #[cfg(feature = "regex")]
            "regexpIsRegexp" => {
                linker.func_wrap("dart", import.name(), regex::func_regexp_is_regexp)?;
            }
            #[cfg(feature = "regex")]
            "regexpEscape" => {
                linker.func_wrap("dart", import.name(), regex::func_regexp_escape)?;
            }
            #[cfg(feature = "regex")]
            "regexpMatch" => {
                linker.func_wrap("dart", import.name(), regex::func_regexp_match)?;
            }
            #[cfg(feature = "regex")]
            "regexpMatchGetStart" => {
                linker.func_wrap("dart", import.name(), regex::func_regexp_match_get_start)?;
            }
            #[cfg(feature = "regex")]
            "regexpMatchGetEnd" => {
                linker.func_wrap("dart", import.name(), regex::func_regexp_match_get_end)?;
            }
            #[cfg(feature = "regex")]
            "regexpMatchGetGroupCount" => {
                linker.func_wrap("dart", import.name(), regex::func_regexp_match_group_count)?;
            }
            #[cfg(feature = "regex")]
            "regexpMatchGetGroup" => {
                linker.func_wrap("dart", import.name(), regex::func_regexp_match_group)?;
            }
            #[cfg(feature = "regex")]
            "regexpMatchGetNamedGroups" => {
                linker.func_wrap(
                    "dart",
                    import.name(),
                    regex::func_regexp_match_get_named_groups,
                )?;
            }
            #[cfg(feature = "regex")]
            "regexpMatchGetGroupName" => {
                linker.func_wrap(
                    "dart",
                    import.name(),
                    regex::func_regexp_match_get_group_name,
                )?;
            }
            #[cfg(feature = "regex")]
            "regexpMatchGetGroupByName" => {
                linker.func_wrap(
                    "dart",
                    import.name(),
                    regex::func_regexp_match_get_group_by_name,
                )?;
            }
            _ => bail!("Unknown Dart import: {}", import.name()),
        }
    }

    Ok(())
}

pub fn invoke_main(instance: &Instance, store: &mut impl AsContextMut) -> Result<()> {
    let func = instance
        .get_func(&mut *store, "$invokeMain")
        .ok_or_else(|| format_err!("Missing $invokeMain export"))?;
    let ValType::Ref(r) = func.ty(&mut *store).param(0).unwrap() else {
        bail!("Expected $invokeMain to take a reftype")
    };
    let HeapType::ConcreteArray(arr_type) = r.heap_type() else {
        bail!("Expected $invokeMain to take an array")
    };

    let allocator = ArrayRefPre::new(&mut *store, arr_type.clone());
    let array = ArrayRef::new(&mut *store, &allocator, &Val::I32(0), 0)?;
    let array: Rooted<AnyRef> = array.into();

    func.call(&mut *store, &[Val::AnyRef(Some(array))], &mut [])?;
    Ok(())
}
