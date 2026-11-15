//! Shared reflected type resolution for native recipe observations.
use enshwick::resources::{Kind, Registry, Type};
use policy::client_recipe as gate;

pub(super) fn field<'a>(
    registry: &'a Registry<'a>,
    ty: &'a Type<'a>,
    name: &str,
) -> Result<(usize, &'a Type<'a>), String> {
    let ty = concrete(registry, ty)?;
    let field = ty
        .fields()
        .iter()
        .find(|field| field.name() == name)
        .ok_or("recipe reflected field unavailable")?;
    let inner = registry
        .named(field.type_name().ok_or("recipe field type unavailable")?)
        .ok_or("recipe type unavailable")?;
    gate::field(
        field.offset() as usize,
        inner.size() as usize,
        ty.size() as usize,
    )?;
    Ok((field.offset() as usize, inner))
}

pub(super) fn concrete<'a>(
    registry: &'a Registry<'a>,
    mut ty: &'a Type<'a>,
) -> Result<&'a Type<'a>, String> {
    for _ in 0..registry.len() {
        if ty.kind() != Kind::Typedef {
            return Ok(ty);
        }
        let inner = registry
            .named(ty.inner().ok_or("recipe alias target unavailable")?)
            .ok_or("recipe alias type unavailable")?;
        if inner.size() != ty.size() {
            return Err("recipe alias size mismatch".into());
        }
        ty = inner;
    }
    Err("cyclic recipe alias".into())
}

pub(super) fn element<'a>(
    registry: &'a Registry<'a>,
    ty: &Type<'a>,
) -> Result<&'a Type<'a>, String> {
    if ty.kind() != Kind::BlobArray || ty.size() != 8 {
        return Err("recipe field is not a reflected blob array".into());
    }
    registry
        .named(ty.inner().ok_or("recipe array element unavailable")?)
        .ok_or("recipe array type unavailable".into())
}
