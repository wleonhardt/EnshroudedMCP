//! Bounded vanilla archive observations; raw recipe fields are not eligibility.

use enshwick::content::{Blob, Guid, recipes::RecipeLayout};
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

pub(crate) fn parse(
    layout: RecipeLayout,
    bytes: &[u8],
    output: Guid,
    limit: usize,
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
                results.push(
                    json!({"recipe_guid":guid(&blob,at+layout.guid)?.to_string(),"debug_name":name,
                    "workshop_guid":guid(&blob,at+layout.workshop_guid)?.to_string(),
                    "inputs":values(&inputs),"outputs":values(&outputs),
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
}
