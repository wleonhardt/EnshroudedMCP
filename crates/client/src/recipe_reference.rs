//! Cross-check published SDK workshop fields against canonical recipe reflection.
use crate::recipe_types::{concrete, element, field};
use enshwick::{
    content::recipes::RecipeLayout,
    resources::{Kind, Registry},
};
use policy::client_recipe::ReferenceLayout;

pub(crate) fn check(
    registry: &Registry<'_>,
    hash: u32,
    layout: RecipeLayout,
) -> Result<(), String> {
    let root = registry
        .hashed(hash)
        .ok_or("recipe registry type unavailable")?;
    let (array, recipes) = field(registry, root, "recipes")?;
    let recipe = element(registry, recipes)?;
    let (guid, guid_type) = field(registry, recipe, "workshopGuid")?;
    let guid_type = concrete(registry, guid_type)?;
    let (id, id_type) = field(registry, recipe, "workshopId")?;
    let id_type = concrete(registry, id_type)?;
    let (value, scalar) = if id_type.kind() == Kind::Uint32 {
        (0, id_type)
    } else {
        field(registry, id_type, "value")?
    };
    let scalar = concrete(registry, scalar)?;
    ReferenceLayout {
        array: layout.recipes,
        record: layout.recipe,
        guid: layout.workshop_guid,
        id: layout.workshop,
    }
    .check(
        ReferenceLayout {
            array,
            record: recipe.size() as usize,
            guid,
            id: id
                .checked_add(value)
                .ok_or("recipe workshop ID offset overflow")?,
        },
        guid_type.kind() == Kind::Guid && guid_type.size() == 16,
        scalar.kind() == Kind::Uint32 && scalar.size() == 4,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::resources::{Field, Type};
    fn registry(wrapped: bool, kind: Kind, scalar_kind: Kind) -> Registry<'static> {
        Registry::new([
            Type::new("q::u32", scalar_kind, 4, 4),
            Type::new("q::guid", kind, 16, 4),
            Type::new("q::Id", Kind::Struct, 4, 4)
                .with_fields(vec![Field::new("value", 0, "q::u32")]),
            Type::new("q::Alias", Kind::Typedef, 4, 4).with_inner("q::Id"),
            Type::new("q::Recipe", Kind::Struct, 48, 4).with_fields(vec![
                Field::new("workshopGuid", 8, "q::guid"),
                Field::new(
                    "workshopId",
                    32,
                    if wrapped { "q::Alias" } else { "q::u32" },
                ),
            ]),
            Type::new("q::Rows", Kind::BlobArray, 8, 4).with_inner("q::Recipe"),
            Type::new("q::Registry", Kind::Struct, 8, 4)
                .with_fields(vec![Field::new("recipes", 0, "q::Rows")]),
        ])
    }
    fn layout() -> RecipeLayout {
        RecipeLayout {
            recipes: 0,
            recipe: 48,
            guid: 0,
            id: 0,
            workshop_guid: 8,
            workshop: 32,
            input: 0,
            output: 0,
            completion: 0,
            relative: [0; 4],
            input_size: 24,
            stack_size: 24,
            stack_item: 16,
            stack_count: 20,
        }
    }
    #[test]
    fn canonical_scalar_and_alias_contracts_agree_without_reading_or_filling_values() {
        for wrapped in [true, false] {
            let r = registry(wrapped, Kind::Guid, Kind::Uint32);
            let hash = r.named("q::Registry").unwrap().qualified_hash();
            assert!(check(&r, hash, layout()).is_ok());
            assert!(
                check(
                    &r,
                    hash,
                    RecipeLayout {
                        workshop_guid: 12,
                        ..layout()
                    }
                )
                .is_err()
            );
            assert!(
                check(
                    &r,
                    hash,
                    RecipeLayout {
                        workshop: 36,
                        ..layout()
                    }
                )
                .is_err()
            );
            assert!(
                check(
                    &r,
                    hash,
                    RecipeLayout {
                        recipe: 64,
                        ..layout()
                    }
                )
                .is_err()
            );
            assert!(check(&r, 0, layout()).is_err());
        }
        let r = registry(true, Kind::Guid, Kind::Float32);
        assert!(
            check(
                &r,
                r.named("q::Registry").unwrap().qualified_hash(),
                layout()
            )
            .is_err()
        );
        let r = registry(true, Kind::Struct, Kind::Uint32);
        assert!(
            check(
                &r,
                r.named("q::Registry").unwrap().qualified_hash(),
                layout()
            )
            .is_err()
        );
    }
}
