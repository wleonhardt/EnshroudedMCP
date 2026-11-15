//! Declared vanilla prop/shelter requirements, not current eligibility.

use crate::recipe_types::{concrete, element, field};
use enshwick::content::Blob;
use enshwick::resources::{Kind, Registry};
use policy::{client_recipe as gate, item_lookup::span};
use serde_json::{Value, json};

pub(crate) struct Reader {
    layout: gate::RequirementsLayout,
}

impl Reader {
    pub(crate) fn resolve(
        registry: &Registry<'_>,
        hash: u32,
        expected_record: usize,
    ) -> Result<Self, String> {
        let root = registry
            .hashed(hash)
            .ok_or("recipe registry type unavailable")?;
        let (_, recipes) = field(registry, root, "recipes")?;
        let recipe = element(registry, recipes)?;
        if recipe.size() as usize != expected_record {
            return Err("recipe requirement record size differs from recipe layout".into());
        }
        let (props, array_type) = field(registry, recipe, "requiredProps")?;
        let prop = element(registry, array_type)?;
        let (prop_value, scalar) = field(registry, prop, "value")?;
        let scalar = concrete(registry, scalar)?;
        if scalar.kind() != Kind::Uint32 || scalar.size() != 4 {
            return Err("required prop ID is not a reflected u32".into());
        }
        let (shelter, boolean) = field(registry, recipe, "requiresSheltered")?;
        let boolean = concrete(registry, boolean)?;
        if boolean.kind() != Kind::Bool || boolean.size() != 1 {
            return Err("recipe shelter flag is not a reflected boolean".into());
        }
        Ok(Self {
            layout: gate::RequirementsLayout {
                record: expected_record,
                props,
                prop_stride: prop.size() as usize,
                prop_value,
                shelter,
            },
        })
    }

    pub(crate) fn read(&self, bytes: &[u8], at: usize) -> Result<Value, String> {
        let layout = self.layout;
        layout.check(bytes.len(), at)?;
        let blob = Blob::new(bytes);
        let (start, count) = blob
            .array(at + layout.props)
            .ok_or("malformed required prop array")?;
        gate::count(count as usize, gate::MAX_REQUIRED_PROPS)?;
        span(start, count as usize, layout.prop_stride, bytes.len())?;
        let ids = (0..count as usize)
            .map(|index| {
                blob.read_u32(start + index * layout.prop_stride + layout.prop_value)
                    .ok_or("malformed required prop ID")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let shelter = gate::bool_byte(bytes[at + layout.shelter])?;
        Ok(
            json!({"state":"observed","source":"vanilla_archive_recipe_requirements",
            "required_prop_ids":ids,"requires_shelter":shelter,"full_requirements_verified":false,
            "eligibility_verified":false}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::content::Blob;
    use enshwick::resources::{Field, Kind, Registry, Type};
    use serde_json::json;

    fn registry() -> Registry<'static> {
        Registry::new([
            Type::new("q::u32", Kind::Uint32, 4, 4),
            Type::new("q::bool", Kind::Bool, 1, 1),
            Type::new("q::Id", Kind::Struct, 4, 4)
                .with_fields(vec![Field::new("value", 0, "q::u32")]),
            Type::new("q::PropId", Kind::Typedef, 4, 4).with_inner("q::Id"),
            Type::new("q::Props", Kind::BlobArray, 8, 4).with_inner("q::PropId"),
            Type::new("q::Recipe", Kind::Struct, 32, 4).with_fields(vec![
                Field::new("requiredProps", 8, "q::Props"),
                Field::new("requiresSheltered", 28, "q::bool"),
            ]),
            Type::new("q::Recipes", Kind::BlobArray, 8, 4).with_inner("q::Recipe"),
            Type::new("q::Registry", Kind::Struct, 8, 4).with_fields(vec![Field::new(
                "recipes",
                0,
                "q::Recipes",
            )]),
        ])
    }

    fn fixture(ids: &[u32], shelter: bool) -> Vec<u8> {
        let mut blob = Blob::new(&[0; 64]);
        let start = blob.append(&vec![0; ids.len() * 4], 4);
        assert!(blob.set_array(24, start, ids.len() as u32));
        for (index, id) in ids.iter().enumerate() {
            assert!(blob.write(start + index * 4, &id.to_le_bytes()));
        }
        assert!(blob.write(44, &[u8::from(shelter)]));
        blob.into_bytes()
    }

    #[test]
    fn reflected_requirements_preserve_declared_order_empty_and_false() {
        let registry = registry();
        let reader = Reader::resolve(
            &registry,
            registry.named("q::Registry").unwrap().qualified_hash(),
            32,
        )
        .unwrap();
        for (ids, shelter) in [(vec![], false), (vec![0, 9, 9], true)] {
            let value = reader.read(&fixture(&ids, shelter), 16).unwrap();
            assert_eq!(value["required_prop_ids"], json!(ids));
            assert_eq!(value["requires_shelter"], shelter);
            assert_eq!(value["full_requirements_verified"], false);
            assert_eq!(value["state"], "observed");
        }
        assert!(Reader::resolve(&registry, 0, 32).is_err());
        assert!(
            Reader::resolve(
                &registry,
                registry.named("q::Registry").unwrap().qualified_hash(),
                31
            )
            .is_err()
        );
    }

    #[test]
    fn malformed_or_excessive_prop_spans_refuse_before_observation() {
        let registry = registry();
        let reader = Reader::resolve(
            &registry,
            registry.named("q::Registry").unwrap().qualified_hash(),
            32,
        )
        .unwrap();
        let bytes = fixture(&[7, 8], false);
        assert!(reader.read(&bytes[..bytes.len() - 1], 16).is_err());
        assert!(reader.read(&bytes, usize::MAX).is_err());
        let mut bad = Blob::new(&bytes);
        let (start, _) = bad.array(24).unwrap();
        assert!(bad.set_array(24, start, 33));
        assert!(reader.read(bad.bytes(), 16).is_err());
    }
}
