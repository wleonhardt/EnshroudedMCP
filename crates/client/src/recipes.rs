//! Bounded vanilla archive observations; raw recipe fields are not eligibility.

use enshwick::content::{Blob, Guid, recipes::RecipeLayout};
use enshwick::resources::Registry;
use enshwick::typed::{RecipeInputResource, Reflected};
use policy::{client_recipe as gate, item_lookup::span};
use serde_json::{Value, json};

fn array(
    blob: &Blob,
    field: usize,
    maximum: usize,
    stride: usize,
) -> Result<(usize, usize), String> {
    span(field, 1, 8, blob.bytes().len())?;
    let (start, count) = blob.array(field).ok_or("malformed recipe array header")?;
    let count = count as usize;
    gate::count(count, maximum)?;
    span(start, count, stride, blob.bytes().len())?;
    Ok((start, count))
}

fn guid(blob: &Blob, at: usize) -> Result<Guid, String> {
    let range = span(at, 1, 16, blob.bytes().len())?;
    Ok(Guid(
        blob.bytes()[range]
            .try_into()
            .map_err(|_| "malformed recipe GUID")?,
    ))
}

fn stacks(
    blob: &Blob,
    field: usize,
    maximum: usize,
    stride: usize,
    count_offset: usize,
) -> Result<Vec<(Guid, u32)>, String> {
    let (start, count) = array(blob, field, maximum, stride)?;
    (0..count)
        .map(|i| {
            let at = start + i * stride; // the complete span and field offsets are admitted first
            Ok((
                guid(blob, at)?,
                blob.read_u32(at + count_offset)
                    .ok_or("malformed recipe stack count")?,
            ))
        })
        .collect()
}

fn values(stacks: &[(Guid, u32)]) -> Vec<Value> {
    stacks
        .iter()
        .map(|(item, count)| json!({"item_guid":item.to_string(),"count":count}))
        .collect()
}

fn decode_input(
    registry: &Registry<'_>,
    input_type_hash: u32,
    bytes: &[u8],
) -> Result<Value, String> {
    let name = registry
        .hashed(input_type_hash)
        .ok_or("recipe input reflected type unavailable")?
        .qualified_name();
    let value = registry.decode(name, bytes).map_err(|e| e.to_string())?;
    let input = RecipeInputResource::from_value(&value).map_err(|e| e.to_string())?;
    let category_id = input.input_item_category.category.value;
    Ok(
        json!({"state":"observed","source":"published_reflected_recipe_input_resource",
        "item_guid":input.item_stack.item_ref.to_string(),"item_id":input.item_stack.item.value,"item_count":input.item_stack.count,
        "category_guid":input.input_item_category.category_ref.to_string(),
        "category_id":category_id,"category_count":input.input_item_category.count,
        "branch_semantics_verified":false}),
    )
}

pub(crate) fn parse(
    layout: RecipeLayout,
    bytes: &[u8],
    output: Guid,
    limit: usize,
    input_type_hash: u32,
) -> Result<Value, String> {
    gate::request(bytes.len(), limit)?;
    let registry = Registry::game().map_err(|e| e.to_string())?;
    parse_with(layout, bytes, output, limit, |record| {
        decode_input(registry, input_type_hash, record)
    })
}

fn parse_with(
    layout: RecipeLayout,
    bytes: &[u8],
    output: Guid,
    limit: usize,
    mut decode: impl FnMut(&[u8]) -> Result<Value, String>,
) -> Result<Value, String> {
    gate::request(bytes.len(), limit)?;
    for size in [layout.recipe, layout.input_size, layout.stack_size] {
        gate::stride(size)?;
    }
    for (offset, size) in [
        (layout.guid, 16),
        (layout.id, 4),
        (layout.workshop_guid, 16),
        (layout.workshop, 4),
        (layout.input, 8),
        (layout.output, 8),
        (layout.completion, 4),
    ] {
        gate::field(offset, size, layout.recipe)?;
    }
    for offset in layout.relative {
        gate::field(offset, 8, layout.recipe)?;
    }
    for size in [layout.input_size, layout.stack_size] {
        gate::field(0, 16, size)?;
        gate::field(layout.stack_item, 4, size)?;
        gate::field(layout.stack_count, 4, size)?;
    }
    let blob = Blob::new(bytes);
    let (start, count) = array(&blob, layout.recipes, gate::MAX_RECIPES, layout.recipe)?;
    let mut results = Vec::new();
    let mut total = 0;
    for i in 0..count {
        let at = start + i * layout.recipe;
        let (name_at, name_len) = array(&blob, at + layout.relative[3], 128, 1)?;
        gate::name(name_len)?;
        let name = std::str::from_utf8(&bytes[name_at..name_at + name_len])
            .map_err(|_| "recipe name is not UTF-8")?;
        let inputs = stacks(
            &blob,
            at + layout.input,
            gate::MAX_INPUTS,
            layout.input_size,
            layout.stack_count,
        )?;
        let outputs = stacks(
            &blob,
            at + layout.output,
            gate::MAX_OUTPUTS,
            layout.stack_size,
            layout.stack_count,
        )?;
        if outputs.iter().any(|(item, _)| *item == output) {
            total += 1;
            if results.len() < limit {
                let (input_start, input_count) = array(
                    &blob,
                    at + layout.input,
                    gate::MAX_INPUTS,
                    layout.input_size,
                )?;
                let input_observations = (0..input_count)
                    .map(|index| {
                        let start = input_start + index * layout.input_size;
                        decode(&bytes[start..start + layout.input_size])
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                results.push(
                    json!({"recipe_guid":guid(&blob,at+layout.guid)?.to_string(),"debug_name":name,
                    "workshop_guid":guid(&blob,at+layout.workshop_guid)?.to_string(),
                    "inputs":values(&inputs),"input_observations":input_observations,"outputs":values(&outputs),
                    "ingredient_category_semantics":null,"catalyst_semantics":null,
                    "required_props":null,"unlock":null,"feasible":null}),
                );
            }
        }
    }
    Ok(
        json!({"mode":"report_only","source":"vanilla_archive_recipe_registry","effective_definition_verified":false,
        "selected_output_guid":output.to_string(),"registry_entries":count,"total_matches":total,
        "truncated":total>results.len(),"recipes":results,
        "unverified":["ingredient category and catalyst semantics","required props and full unlock conditions","station availability and player eligibility","effective mod and settings changes","crafting feasibility and survival costs"],
        "limitations":["whole upstream registry allocation precedes parser byte admission","all matching output positions are considered; raw inputs/outputs do not establish recipe semantics"]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(
        layout: RecipeLayout,
        bytes: &[u8],
        output: Guid,
        limit: usize,
    ) -> Result<Value, String> {
        super::parse_with(layout, bytes, output, limit, |_| {
            Ok(json!({"state":"synthetic_unavailable"}))
        })
    }

    // Synthetic records, not offsets copied from a game build.
    fn layout() -> RecipeLayout {
        RecipeLayout {
            recipes: 0,
            recipe: 104,
            guid: 0,
            id: 16,
            workshop_guid: 24,
            workshop: 40,
            input: 48,
            output: 56,
            completion: 64,
            relative: [48, 56, 72, 80],
            input_size: 24,
            stack_size: 24,
            stack_item: 16,
            stack_count: 20,
        }
    }

    fn registry(count: usize) -> (Vec<u8>, Guid) {
        let layout = layout();
        let selected = Guid([3; 16]);
        let mut blob = Blob::new(&[0; 8]);
        let start = blob.append(&vec![0; count * layout.recipe], 8);
        assert!(blob.set_array(0, start, count as u32));
        for i in 0..count {
            let at = start + i * layout.recipe;
            assert!(blob.write(at + layout.guid, &Guid([1; 16]).0));
            assert!(blob.write(at + layout.workshop_guid, &Guid([4; 16]).0));
            for (field, items) in [
                (layout.input, vec![(Guid([2; 16]), 3u32)]),
                (layout.output, vec![(Guid([5; 16]), 1u32), (selected, 2u32)]),
            ] {
                let stacks = blob.append(&vec![0; items.len() * 24], 8);
                assert!(blob.set_array(at + field, stacks, items.len() as u32));
                for (index, (item, count)) in items.iter().enumerate() {
                    assert!(blob.write(stacks + index * 24, &item.0));
                    assert!(blob.write(stacks + index * 24 + 20, &count.to_le_bytes()));
                }
            }
            let name = blob.append(b"fixture-recipe", 1);
            assert!(blob.set_array(at + 80, name, 14));
        }
        (blob.into_bytes(), selected)
    }

    #[test]
    fn second_output_matches_and_all_raw_stacks_survive() {
        let (bytes, selected) = registry(1);
        let out = parse(layout(), &bytes, selected, 8).unwrap();
        assert_eq!(out["total_matches"], 1);
        let recipe = &out["recipes"][0];
        assert_eq!(recipe["recipe_guid"], Guid([1; 16]).to_string());
        assert_eq!(recipe["workshop_guid"], Guid([4; 16]).to_string());
        assert_eq!(recipe["debug_name"], "fixture-recipe");
        assert_eq!(recipe["inputs"][0]["count"], 3);
        assert_eq!(recipe["outputs"].as_array().unwrap().len(), 2);
        assert_eq!(recipe["outputs"][1]["count"], 2);
        assert_eq!(recipe["outputs"][1]["item_guid"], selected.to_string());
        assert!(recipe["unlock"].is_null() && recipe["feasible"].is_null());
        assert_eq!(out["effective_definition_verified"], false);
        assert_eq!(
            parse(layout(), &bytes, Guid([7; 16]), 8).unwrap()["total_matches"],
            0
        );
    }

    #[test]
    fn matching_results_are_capped_without_hiding_total() {
        let (bytes, selected) = registry(10);
        let out = parse(layout(), &bytes, selected, 2).unwrap();
        assert_eq!(out["total_matches"], 10);
        assert_eq!(out["recipes"].as_array().unwrap().len(), 2);
        assert_eq!(out["truncated"], true);
    }

    #[test]
    fn malformed_arrays_counts_layouts_and_truncation_are_refused() {
        let (bytes, selected) = registry(1);
        for end in [0, 4, 8, bytes.len() - 1] {
            assert!(parse(layout(), &bytes[..end], selected, 1).is_err());
        }
        for (field, value) in [(0, 16385), (8 + 48, 33), (8 + 56, 9), (8 + 80, 129)] {
            let mut bad = Blob::new(&bytes);
            let (start, _) = bad.array(field).unwrap();
            assert!(bad.set_array(field, start, value));
            assert!(parse(layout(), bad.bytes(), selected, 1).is_err());
        }
        let mut invalid = layout();
        invalid.recipes = usize::MAX;
        assert!(parse(invalid, &bytes, selected, 1).is_err());
        let mut invalid = layout();
        invalid.stack_count = usize::MAX;
        assert!(parse(invalid, &bytes, selected, 1).is_err());
        let mut invalid = layout();
        invalid.guid = 100;
        assert!(parse(invalid, &bytes, selected, 1).is_err());
        let mut invalid = layout();
        invalid.input_size = 0;
        assert!(parse(invalid, &bytes, selected, 1).is_err());
        let mut invalid = layout();
        invalid.relative[3] = usize::MAX;
        assert!(parse(invalid, &bytes, selected, 1).is_err());
        let mut bad = Blob::new(&bytes);
        assert!(bad.write(0, &i32::MIN.to_le_bytes()));
        assert!(parse(layout(), bad.bytes(), selected, 1).is_err());
        let mut bad = bytes;
        *bad.last_mut().unwrap() = 255;
        assert!(parse(layout(), &bad, selected, 1).is_err());
    }
    #[test]
    fn category_only_input_is_preserved_without_claiming_free_crafting() {
        let (bytes, selected) = registry(1);
        let mut blob = Blob::new(&bytes);
        let row = blob.append(&[0; 48], 8);
        assert!(blob.set_array(8 + layout().input, row, 1));
        assert!(blob.write(row + 24, &Guid([9; 16]).0));
        assert!(blob.write(row + 44, &5u32.to_le_bytes()));
        let mut layout = layout();
        layout.input_size = 48;
        let result = parse_with(layout, blob.bytes(), selected, 1, |bytes| {
            assert_eq!(bytes.len(), 48);
            assert_eq!(&bytes[24..40], &Guid([9; 16]).0);
            Ok(json!({"state":"observed","category_guid":Guid([9;16]).to_string(),"category_id":7,"category_count":5}))
        }).unwrap();
        let recipe = &result["recipes"][0];
        assert_eq!(recipe["inputs"][0]["count"], 0);
        assert_eq!(recipe["input_observations"][0]["category_count"], 5);
        assert!(recipe["ingredient_category_semantics"].is_null());
        assert!(recipe["feasible"].is_null());
    }

    fn input_registry() -> Registry<'static> {
        input_registry_named("RecipeInputResource")
    }

    fn input_registry_named(name: &'static str) -> Registry<'static> {
        use enshwick::resources::{Field, Kind, Type};
        Registry::new([
            Type::new("u32", Kind::Uint32, 4, 4),
            Type::new("HashKey32", Kind::Struct, 4, 4)
                .with_fields(vec![Field::new("value", 0, "u32")]),
            Type::new("Guid", Kind::Guid, 16, 4),
            Type::new("RecipeItemStackResource", Kind::Struct, 24, 4).with_fields(vec![
                Field::new("itemRef", 0, "Guid"),
                Field::new("item", 16, "HashKey32"),
                Field::new("count", 20, "u32"),
            ]),
            Type::new("RecipeInputCategoryResource", Kind::Struct, 24, 4).with_fields(vec![
                Field::new("categoryRef", 0, "Guid"),
                Field::new("category", 16, "HashKey32"),
                Field::new("count", 20, "u32"),
            ]),
            Type::new(name, Kind::Struct, 48, 4).with_fields(vec![
                Field::new("itemStack", 0, "RecipeItemStackResource"),
                Field::new("inputItemCategory", 24, "RecipeInputCategoryResource"),
            ]),
        ])
    }

    #[test]
    fn published_typed_decoder_preserves_item_category_and_mixed_branches() {
        let registry = input_registry();
        for (item, category) in [(3u32, 0u32), (0, 5), (3, 5)] {
            let mut bytes = [0; 48];
            bytes[..16].copy_from_slice(&Guid([if item == 0 { 0 } else { 2 }; 16]).0);
            bytes[20..24].copy_from_slice(&item.to_le_bytes());
            bytes[24..40].copy_from_slice(&Guid([if category == 0 { 0 } else { 9 }; 16]).0);
            bytes[40..44].copy_from_slice(&7u32.to_le_bytes());
            bytes[44..48].copy_from_slice(&category.to_le_bytes());
            let value = decode_input(
                &registry,
                registry
                    .named("RecipeInputResource")
                    .unwrap()
                    .qualified_hash(),
                &bytes,
            )
            .unwrap();
            assert_eq!(
                value["item_guid"],
                Guid([if item == 0 { 0 } else { 2 }; 16]).to_string()
            );
            assert_eq!(value["item_count"], item);
            assert_eq!(
                value["category_guid"],
                Guid([if category == 0 { 0 } else { 9 }; 16]).to_string()
            );
            assert_eq!(value["category_id"], 7);
            assert_eq!(value["category_count"], category);
            assert_eq!(value["branch_semantics_verified"], false);
        }
        assert!(
            decode_input(
                &registry,
                registry
                    .named("RecipeInputResource")
                    .unwrap()
                    .qualified_hash(),
                &[0; 47]
            )
            .is_err()
        );
        assert!(decode_input(&Registry::new([]), 7, &[0; 48]).is_err());
    }

    #[test]
    fn decoding_only_returned_matches_and_refusing_oversized_inputs_before_decoder() {
        let (bytes, selected) = registry(10);
        let mut calls = 0;
        parse_with(layout(), &bytes, selected, 2, |_| {
            calls += 1;
            Ok(Value::Null)
        })
        .unwrap();
        assert_eq!(calls, 2);
        parse_with(layout(), &bytes, Guid([8; 16]), 2, |_| {
            panic!("unmatched recipe decoded")
        })
        .unwrap();
        let mut bad = Blob::new(&bytes);
        let (start, _) = bad.array(8 + layout().input).unwrap();
        assert!(bad.set_array(8 + layout().input, start, (gate::MAX_INPUTS + 1) as u32));
        assert!(
            parse_with(layout(), bad.bytes(), selected, 1, |_| panic!(
                "oversized input decoded"
            ))
            .is_err()
        );
        assert!(
            parse_with(layout(), &bytes, selected, 1, |_| Err(
                "typed input unavailable".into()
            ))
            .is_err()
        );
    }
    #[test]
    fn qualified_input_type_does_not_require_a_bare_type_name() {
        let registry = input_registry_named("fixture::RecipeInputResource");
        assert!(registry.named("RecipeInputResource").is_none());
        assert!(
            decode_input(
                &registry,
                registry
                    .named("fixture::RecipeInputResource")
                    .unwrap()
                    .qualified_hash(),
                &[0; 48]
            )
            .is_ok()
        );
    }
}
