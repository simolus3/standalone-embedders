use alloc::vec::Vec;

use wasmtime::{AnyRef, AsContext, Caller, ExternRef, OwnedRooted, Result, Rooted, Val};

use crate::{
    DartEmbedder,
    utils::{externref_mut, externref_ref, null_check},
};

use hashbrown::HashMap;

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

/// A map from Dart objects to values.
#[derive(Default)]
struct Expando {
    // keyed by dart identity hash code (randomized)
    entries: HashMap<i64, Vec<ExpandoEntry>>,
}

struct ExpandoEntry {
    key: OwnedRooted<AnyRef>,
    value: OwnedRooted<AnyRef>,
}

impl Expando {
    fn find_index(
        &self,
        store: impl AsContext,
        target: &Rooted<AnyRef>,
        target_identity_hash_code: i64,
    ) -> Result<Option<usize>> {
        let Some(bucket) = self.entries.get(&target_identity_hash_code) else {
            return Ok(None);
        };

        let store = store.as_context();
        for (i, entry) in bucket.iter().enumerate() {
            if Rooted::ref_eq(&store, &entry.key, target)? {
                return Ok(Some(i));
            }
        }

        Ok(None)
    }
}

pub fn func_expando_create<E: DartEmbedder>(caller: Caller<'_, E>) -> Result<Rooted<ExternRef>> {
    ExternRef::new(caller, Expando::default())
}

pub fn func_expando_get<E: DartEmbedder>(
    mut caller: Caller<'_, E>,
    expando: Option<Rooted<ExternRef>>,
    target: Option<Rooted<AnyRef>>,
    target_identity_hash_code: i64,
) -> Result<Option<Rooted<AnyRef>>> {
    let expando = null_check(expando)?;
    let target = null_check(target)?;

    let value = {
        let data: &Expando = externref_ref(&caller, &expando)?;
        data.find_index(&caller, &target, target_identity_hash_code)?
            .map(|i| data.entries[&target_identity_hash_code][i].value.clone())
    };

    Ok(value.map(|value| value.to_rooted(&mut caller)))
}

pub fn func_expando_set<E: DartEmbedder>(
    mut caller: Caller<'_, E>,
    expando: Option<Rooted<ExternRef>>,
    target: Option<Rooted<AnyRef>>,
    target_identity_hash_code: i64,
    value: Option<Rooted<AnyRef>>,
) -> Result<()> {
    let expando = null_check(expando)?;
    let target = null_check(target)?;

    let index = externref_ref::<_, Expando>(&caller, &expando)?.find_index(
        &caller,
        &target,
        target_identity_hash_code,
    )?;

    match value {
        Some(value) => {
            let value = value.to_owned_rooted(&mut caller)?;
            match index {
                Some(i) => {
                    let data: &mut Expando = externref_mut(&mut caller, &expando)?;
                    data.entries.get_mut(&target_identity_hash_code).unwrap()[i].value = value;
                }
                None => {
                    let key = target.to_owned_rooted(&mut caller)?;
                    let data: &mut Expando = externref_mut(&mut caller, &expando)?;
                    data.entries
                        .entry(target_identity_hash_code)
                        .or_default()
                        .push(ExpandoEntry { key, value });
                }
            }
        }
        None => {
            // Setting null removes the entry.
            if let Some(i) = index {
                let data: &mut Expando = externref_mut(&mut caller, &expando)?;
                let bucket = data.entries.get_mut(&target_identity_hash_code).unwrap();
                bucket.swap_remove(i);
                if bucket.is_empty() {
                    data.entries.remove(&target_identity_hash_code);
                }
            }
        }
    }

    Ok(())
}

// Since we don't have weak references, objects attached to finalizers never become unreachable
// while the store is alive. So finalizer callbacks are never invoked, and we don't need to keep
// track of attached objects at all.

struct Finalizer;

/// `finalizerCreate`, implemented with `func_new` since the callback parameter is a typed funcref.
pub fn func_finalizer_create<E: DartEmbedder>(
    caller: Caller<'_, E>,
    _args: &[Val],
    results: &mut [Val],
) -> Result<()> {
    results[0] = Val::ExternRef(Some(ExternRef::new(caller, Finalizer)?));
    Ok(())
}

pub fn func_finalizer_attach<E: DartEmbedder>(
    _caller: Caller<'_, E>,
    finalizer: Option<Rooted<ExternRef>>,
    object: Option<Rooted<AnyRef>>,
    _token: Option<Rooted<AnyRef>>,
    _detach_token: Option<Rooted<AnyRef>>,
) -> Result<()> {
    null_check(finalizer)?;
    null_check(object)?;
    Ok(())
}

pub fn func_finalizer_detach<E: DartEmbedder>(
    _caller: Caller<'_, E>,
    finalizer: Option<Rooted<ExternRef>>,
    detach_token: Option<Rooted<AnyRef>>,
) -> Result<()> {
    null_check(finalizer)?;
    null_check(detach_token)?;
    Ok(())
}
