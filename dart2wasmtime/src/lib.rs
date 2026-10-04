#![no_std]
extern crate alloc;

use core::time::Duration;

use alloc::{boxed::Box, vec};
use wasmtime::{
    AnyRef, ArrayRef, ArrayRefPre, AsContextMut, Caller, ExternRef, ExternType, HeapType, Instance,
    Linker, Module, Result, Rooted, Val, ValType, bail, format_err,
};

use crate::{
    stack_trace::StackTrace,
    string::{DartString, StringBuffer},
    utils::{null_check, null_check_ref},
};

pub use event_loop::{DartCallback, DartSchedule};

mod event_loop;
mod stack_trace;
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
                    |mut caller: Caller<'_, E>, args: &[Val], results: &mut [Val]| {
                        let char_codes =
                            null_check_ref(args[0].unwrap_anyref())?.unwrap_array(&caller)?;
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
                    },
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
            "stackTraceGetCurrent" => {
                linker.func_wrap("dart", import.name(), StackTrace::func_current)?;
            }
            "stackTraceToString" => {
                linker.func_wrap("dart", import.name(), StackTrace::func_to_string)?;
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
