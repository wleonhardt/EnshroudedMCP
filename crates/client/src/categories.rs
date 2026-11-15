//! Vanilla recipe category membership; scaling is observed, not interpreted.

use enshwick::content::Blob;
use enshwick::resources::{Kind, Registry, Type};
use enshwick::typed::{RecipeInputCategoryItem, Reflected};
use gate::CategoryLayout as Layout;
use policy::{client_recipe as gate, item_lookup::span};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn array(
    blob: &Blob,
    field: usize,
    maximum: usize,
    stride: usize,
) -> Result<(usize, usize), String> {
    span(field, 1, 8, blob.bytes().len())?;
    let (start, count) = blob.array(field).ok_or("malformed category array")?;
    gate::count(count as usize, maximum)?;
    span(start, count as usize, stride, blob.bytes().len())?;
    Ok((start, count as usize))
}

struct Index<'a> {
    layout: Layout,
    bytes: &'a [u8],
    rows: BTreeMap<u32, (usize, usize)>,
}

impl<'a> Index<'a> {
    fn new(layout: Layout, bytes: &'a [u8]) -> Result<Self, String> {
        layout.check(bytes.len())?;
        let blob = Blob::new(bytes);
        let (start, count) = array(
            &blob,
            layout.categories,
            gate::MAX_CATEGORIES,
            layout.category_size,
        )?;
        let mut rows = BTreeMap::new();
        for index in 0..count {
            let at = start + index * layout.category_size;
            let id = blob
                .read_u32(at + layout.id)
                .ok_or("malformed category ID")?;
            let members = array(
                &blob,
                at + layout.items,
                gate::MAX_CATEGORY_MEMBERS,
                layout.member_size,
            )?;
            if rows.insert(id, members).is_some() {
                return Err("duplicate recipe category ID".into());
            }
        }
        Ok(Self {
            layout,
            bytes,
            rows,
        })
    }

    fn report(
        &self,
        id: u32,
        mut decode: impl FnMut(&[u8]) -> Result<Value, String>,
    ) -> Result<Value, String> {
        if id == 0 {
            return Ok(json!({"state":"no_category_id","members":null}));
        }
        let Some(&(start, count)) = self.rows.get(&id) else {
            return Ok(json!({"state":"not_found","members":null}));
        };
        let members = (0..count.min(gate::RETURNED_CATEGORY_MEMBERS))
            .map(|index| {
                let at = start + index * self.layout.member_size;
                decode(&self.bytes[at..at + self.layout.member_size])
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(
            json!({"state":"observed","source":"vanilla_archive_recipe_input_categories","members":members,
            "total_members":count,"truncated":count>gate::RETURNED_CATEGORY_MEMBERS,"scaling_semantics_verified":false}),
        )
    }
}

fn field<'a>(
    registry: &'a Registry<'a>,
    ty: &'a Type<'a>,
    name: &str,
) -> Result<(usize, &'a Type<'a>), String> {
    let ty = concrete(registry, ty)?;
    let field = ty
        .fields()
        .iter()
        .find(|field| field.name() == name)
        .ok_or("recipe category reflected field unavailable")?;
    let inner = registry
        .named(
            field
                .type_name()
                .ok_or("recipe category field type unavailable")?,
        )
        .ok_or("recipe category type unavailable")?;
    gate::field(
        field.offset() as usize,
        inner.size() as usize,
        ty.size() as usize,
    )?;
    Ok((field.offset() as usize, inner))
}

fn concrete<'a>(registry: &'a Registry<'a>, mut ty: &'a Type<'a>) -> Result<&'a Type<'a>, String> {
    for _ in 0..registry.len() {
        if ty.kind() != Kind::Typedef {
            return Ok(ty);
        }
        let inner = registry
            .named(
                ty.inner()
                    .ok_or("recipe category alias target unavailable")?,
            )
            .ok_or("recipe category alias type unavailable")?;
        if inner.size() != ty.size() {
            return Err("recipe category alias size mismatch".into());
        }
        ty = inner;
    }
    Err("cyclic recipe category alias".into())
}

fn element<'a>(registry: &'a Registry<'a>, ty: &Type<'a>) -> Result<&'a Type<'a>, String> {
    if ty.kind() != Kind::BlobArray || ty.size() != 8 {
        return Err("recipe category field is not a reflected blob array".into());
    }
    registry
        .named(
            ty.inner()
                .ok_or("recipe category array element unavailable")?,
        )
        .ok_or("recipe category array type unavailable".into())
}

fn resolve<'a>(registry: &'a Registry<'a>, hash: u32) -> Result<(Layout, &'a Type<'a>), String> {
    let root = registry
        .hashed(hash)
        .ok_or("recipe registry reflected type unavailable")?;
    let (categories, array_type) = field(registry, root, "inputCategories")?;
    let category = element(registry, array_type)?;
    let (id, id_type) = field(registry, category, "inputCategoryId")?;
    let (value, scalar) = field(registry, id_type, "value")?;
    let scalar = concrete(registry, scalar)?;
    if scalar.kind() != Kind::Uint32 || scalar.size() != 4 {
        return Err("recipe category ID is not a reflected u32".into());
    }
    gate::field(value, 4, id_type.size() as usize)?;
    let (items, members) = field(registry, category, "items")?;
    let member = element(registry, members)?;
    Ok((
        Layout {
            categories,
            category_size: category.size() as usize,
            id: id + value,
            items,
            member_size: member.size() as usize,
        },
        member,
    ))
}

pub(crate) fn enrich(
    registry: &Registry<'_>,
    hash: u32,
    bytes: &[u8],
    result: &mut Value,
) -> Result<(), String> {
    let (layout, member) = resolve(registry, hash)?;
    let index = Index::new(layout, bytes)?;
    for recipe in result["recipes"]
        .as_array_mut()
        .ok_or("recipe results unavailable")?
    {
        for input in recipe["input_observations"]
            .as_array_mut()
            .ok_or("recipe inputs unavailable")?
        {
            let id = input["category_id"]
                .as_u64()
                .and_then(|id| u32::try_from(id).ok())
                .ok_or("recipe category ID unavailable")?;
            input["category_membership"] = index.report(id, |record| {
                let value = registry.decode(member.qualified_name(), record).map_err(|error| error.to_string())?;
                let item = RecipeInputCategoryItem::from_value(&value).map_err(|error| error.to_string())?;
                Ok(json!({"item_id":item.item_id.value,"scaling":item.scaling,"scaling_finite":item.scaling.is_finite(),"scaling_bits":item.scaling.to_bits()}))
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::content::Blob;
    use serde_json::json;

    fn fixture(count: usize) -> Vec<u8> {
        let mut blob = Blob::new(&[0; 8]);
        let row = blob.append(&[0; 12], 4);
        assert!(blob.set_array(0, row, 1));
        assert!(blob.write(row, &7u32.to_le_bytes()));
        let members = blob.append(&vec![0; count * 8], 4);
        assert!(blob.set_array(row + 4, members, count as u32));
        for index in 0..count {
            assert!(blob.write(members + index * 8, &(index as u32 + 1).to_le_bytes()));
            assert!(blob.write(members + index * 8 + 4, &0.5f32.to_le_bytes()));
        }
        blob.into_bytes()
    }

    fn layout() -> Layout {
        Layout {
            categories: 0,
            category_size: 12,
            id: 0,
            items: 4,
            member_size: 8,
        }
    }

    #[test]
    fn membership_preserves_order_scaling_and_missing_category_uncertainty() {
        let bytes = fixture(2);
        let index = Index::new(layout(), &bytes).unwrap();
        let mut calls = 0;
        let value = index.report(7, |record| {
            calls += 1;
            Ok(json!({"item_id":u32::from_le_bytes(record[..4].try_into().unwrap()),"scaling":0.5}))
        }).unwrap();
        assert_eq!(calls, 2);
        assert_eq!(value["state"], "observed");
        assert_eq!(value["members"][1]["item_id"], 2);
        assert_eq!(value["members"][0]["scaling"], 0.5);
        assert_eq!(value["scaling_semantics_verified"], false);
        assert_eq!(
            index
                .report(8, |_| panic!("missing category decoded"))
                .unwrap()["state"],
            "not_found"
        );
        assert_eq!(
            index
                .report(0, |_| panic!("zero category decoded"))
                .unwrap()["state"],
            "no_category_id"
        );
    }

    #[test]
    fn returned_members_are_capped_before_decoding() {
        let bytes = fixture(40);
        let index = Index::new(layout(), &bytes).unwrap();
        let mut calls = 0;
        let value = index
            .report(7, |_| {
                calls += 1;
                Ok(json!({"item_id":1,"scaling":1.0}))
            })
            .unwrap();
        assert_eq!(calls, 32);
        assert_eq!(value["total_members"], 40);
        assert_eq!(value["truncated"], true);
    }

    #[test]
    fn malformed_spans_duplicate_ids_and_excessive_counts_refuse() {
        let bytes = fixture(2);
        assert!(Index::new(layout(), &bytes[..bytes.len() - 1]).is_err());
        let mut bad = layout();
        bad.items = usize::MAX;
        assert!(Index::new(bad, &bytes).is_err());
        let mut blob = Blob::new(&bytes);
        let (row, _) = blob.array(0).unwrap();
        let (members, _) = blob.array(row + 4).unwrap();
        assert!(blob.set_array(row + 4, members, 1025));
        assert!(Index::new(layout(), blob.bytes()).is_err());
        let mut duplicate = Blob::new(&[0; 8]);
        let rows = duplicate.append(&[0; 24], 4);
        assert!(duplicate.set_array(0, rows, 2));
        for row in [rows, rows + 12] {
            assert!(duplicate.write(row, &7u32.to_le_bytes()));
            assert!(duplicate.set_array(row + 4, rows, 0));
        }
        assert!(Index::new(layout(), duplicate.bytes()).is_err());
    }

    #[test]
    fn qualified_registry_fields_drive_membership_without_bare_name_lookup() {
        use enshwick::resources::Field;
        let registry = Registry::new([
            Type::new("q::uint32", Kind::Uint32, 4, 4),
            Type::new("q::float", Kind::Float32, 4, 4),
            Type::new("q::HashKey32", Kind::Struct, 4, 4).with_fields(vec![Field::new(
                "value",
                0,
                "q::uint32",
            )]),
            Type::new("q::CategoryId", Kind::Typedef, 4, 4).with_inner("q::HashKey32"),
            Type::new("q::Member", Kind::Struct, 8, 4).with_fields(vec![
                Field::new("itemId", 0, "q::HashKey32"),
                Field::new("scaling", 4, "q::float"),
            ]),
            Type::new("q::Members", Kind::BlobArray, 8, 4).with_inner("q::Member"),
            Type::new("q::Category", Kind::Struct, 12, 4).with_fields(vec![
                Field::new("inputCategoryId", 0, "q::CategoryId"),
                Field::new("items", 4, "q::Members"),
            ]),
            Type::new("q::Categories", Kind::BlobArray, 8, 4).with_inner("q::Category"),
            Type::new("q::Registry", Kind::Struct, 8, 4).with_fields(vec![Field::new(
                "inputCategories",
                0,
                "q::Categories",
            )]),
        ]);
        let hash = registry.named("q::Registry").unwrap().qualified_hash();
        let bytes = fixture(2);
        let mut result =
            json!({"recipes":[{"input_observations":[{"category_id":7},{"category_id":0}]}]});
        enrich(&registry, hash, &bytes, &mut result).unwrap();
        let inputs = &result["recipes"][0]["input_observations"];
        assert_eq!(inputs[0]["category_membership"]["members"][1]["item_id"], 2);
        assert_eq!(
            inputs[0]["category_membership"]["members"][0]["scaling_bits"],
            0.5f32.to_bits()
        );
        assert_eq!(inputs[1]["category_membership"]["state"], "no_category_id");
        assert!(resolve(&registry, 0).is_err());
    }
}
