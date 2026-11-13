//! Narrow vanilla registry identity lookup; never reads effective item instances.

use enshwick::content::{Blob, Guid, items::ItemLayout};
use policy::item_lookup as gate;
use serde_json::{Value, json};

pub(crate) fn tool() -> policy::mcp::Tool {
    policy::mcp::Tool {
        name: "item_lookup",
        description: "Map 1..8 inventory item IDs to vanilla archive GUIDs/debug names. Zero/multiple matches explicit, at most 8 per ID. Reads whole upstream registry before the 16 MiB admission cap; traversal capped at 65536 entries and 128 bytes per name. Does not identify effective modded definitions or prove inventory eligibility.",
        schema: gate::schema(),
    }
}

fn array(blob: &Blob, field: usize) -> Result<(usize, usize), String> {
    gate::span(field, 1, 8, blob.bytes().len())?;
    let (at, count) = blob.array(field).ok_or("invalid registry array header")?;
    Ok((at, count as usize))
}

pub(crate) fn parse(layout: ItemLayout, bytes: &[u8], ids: &[u32]) -> Result<Value, String> {
    gate::registry_limits(bytes.len(), 0, 0)?;
    let blob = Blob::new(bytes);
    let (refs, count) = array(&blob, layout.refs)?;
    let (names, names_count) = array(&blob, layout.names)?;
    gate::registry_limits(bytes.len(), count, names_count)?;
    gate::span(refs, count, 16, bytes.len())?;
    gate::span(names, count, 8, bytes.len())?;
    let mut matches: Vec<Vec<Value>> = ids.iter().map(|_| Vec::new()).collect();
    let mut totals = vec![0usize; ids.len()];
    for index in 0..count {
        let guid_at = refs + index * 16; // complete checked spans above
        let guid = Guid(
            bytes[guid_at..guid_at + 16]
                .try_into()
                .map_err(|_| "invalid GUID")?,
        );
        let (text, len) = array(&blob, names + index * 8)?;
        gate::name_limit(len)?;
        let range = gate::span(text, len, 1, bytes.len())?;
        let name =
            std::str::from_utf8(&bytes[range]).map_err(|_| "registry debug name is not UTF-8")?;
        let id = enshwick::ItemId::from_guid(guid).0;
        if let Some(at) = ids.iter().position(|wanted| *wanted == id) {
            totals[at] += 1;
            if matches[at].len() < gate::MAX_MATCHES {
                matches[at].push(json!({"guid":guid.to_string(),"debug_name":name}));
            }
        }
    }
    let results: Vec<_> = ids.iter().enumerate().map(|(i,id)|json!({
        "item_id":id,"state":match totals[i] {0=>"unmatched",1=>"single_match",_=>"multiple_matches"},
        "total_matches":totals[i],"truncated":totals[i]>gate::MAX_MATCHES,"matches":matches[i]
    })).collect();
    Ok(
        json!({"source":"vanilla_archive_item_registry","effective_definition_verified":false,
        "live_identity_semantics_verified":false,"registry_entries":count,"results":results,
        "limitations":["whole registry allocation occurs upstream before byte admission","vanilla archive excludes effective modded definitions and settings","32-bit item IDs can collide; multiple GUID/name matches are not resolved","an unmatched ID does not prove an item is absent from the running game"]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> ItemLayout {
        ItemLayout {
            id: 0,
            object: 0,
            name: 0,
            description: 0,
            flags: 0,
            pid_entity: 0,
            ui_values: 0,
            icon_image: 0,
            ui_value: [0; 5],
            refs: 0,
            names: 8,
        }
    }

    fn registry(items: &[(Guid, &str)]) -> Vec<u8> {
        let mut blob = Blob::new(&[0; 16]);
        let refs: Vec<u8> = items.iter().flat_map(|(guid, _)| guid.0).collect();
        let start = blob.append(&refs, 4);
        assert!(blob.set_array(0, start, items.len() as u32));
        let names = blob.append(&vec![0; items.len() * 8], 4);
        assert!(blob.set_array(8, names, items.len() as u32));
        for (i, (_, name)) in items.iter().enumerate() {
            let text = blob.append(name.as_bytes(), 1);
            assert!(blob.set_array(names + i * 8, text, name.len() as u32));
        }
        blob.into_bytes()
    }

    #[test]
    fn matching_and_unmatched_ids_are_explicit_and_multiple_matches_capped() {
        let guid = Guid([1; 16]);
        let id = enshwick::ItemId::from_guid(guid).0;
        let bytes = registry(&[(guid, "one"), (Guid([2; 16]), "other")]);
        let out = parse(layout(), &bytes, &[id, 123]).unwrap();
        assert_eq!(out["results"][0]["state"], "single_match");
        assert_eq!(out["results"][0]["matches"][0]["debug_name"], "one");
        assert_eq!(out["results"][1]["state"], "unmatched");
        let bytes = registry(&[(guid, "same-id"); 10]);
        let out = parse(layout(), &bytes, &[id]).unwrap();
        assert_eq!(out["results"][0]["state"], "multiple_matches");
        assert_eq!(out["results"][0]["total_matches"], 10);
        assert_eq!(out["results"][0]["matches"].as_array().unwrap().len(), 8);
        assert_eq!(out["results"][0]["truncated"], true);
    }

    #[test]
    fn distinct_guid_hash_collisions_are_not_resolved_silently() {
        let one = Guid([
            0x10, 0x5e, 0x60, 0x0f, 0x38, 0xdb, 0xc8, 0x43, 0x91, 0xd6, 0xe6, 0x67, 0xca, 0x13,
            0xf7, 0xb1,
        ]);
        let two = Guid([
            0xee, 0xad, 0x75, 0xfd, 0xb2, 0x9d, 0x93, 0xd9, 0x43, 0x68, 0x86, 0x80, 0x3d, 0xce,
            0x5f, 0xc9,
        ]);
        assert_ne!(one, two);
        let id = enshwick::ItemId::from_guid(one).0;
        assert_eq!(id, enshwick::ItemId::from_guid(two).0);
        let out = parse(layout(), &registry(&[(one, "one"), (two, "two")]), &[id]).unwrap();
        assert_eq!(out["results"][0]["state"], "multiple_matches");
        assert_eq!(out["results"][0]["total_matches"], 2);
        assert_ne!(
            out["results"][0]["matches"][0]["guid"],
            out["results"][0]["matches"][1]["guid"]
        );
    }

    #[test]
    fn registered_tool_schema_admits_selectors_and_refuses_unknown_arguments() {
        let tool = tool();
        assert_eq!(tool.name, "item_lookup");
        assert_eq!(tool.schema, gate::schema());
        assert!(policy::args::check(&tool.schema, json!({"ids":[1]}).as_object().unwrap()).is_ok());
        assert!(
            policy::args::check(
                &tool.schema,
                json!({"ids":[1],"unknown":true}).as_object().unwrap()
            )
            .is_err()
        );
    }

    #[test]
    fn malformed_counts_spans_names_and_layouts_are_refused() {
        let bytes = registry(&[(Guid([1; 16]), "valid")]);
        for end in [0, 4, 12, bytes.len() - 1] {
            assert!(parse(layout(), &bytes[..end], &[1]).is_err());
        }
        let mut bad = Blob::new(&bytes);
        let (names, _) = bad.array(8).unwrap();
        assert!(bad.set_array(8, names, 0));
        assert!(parse(layout(), bad.bytes(), &[1]).is_err());
        let mut bad = Blob::new(&bytes);
        assert!(bad.set_array(0, 16, 65537));
        assert!(parse(layout(), bad.bytes(), &[1]).is_err());
        let mut bad_layout = layout();
        bad_layout.names = usize::MAX;
        assert!(parse(bad_layout, &bytes, &[1]).is_err());
        let long = "x".repeat(129);
        assert!(parse(layout(), &registry(&[(Guid([1; 16]), &long)]), &[1]).is_err());
        let mut bad = bytes;
        *bad.last_mut().unwrap() = 255;
        assert!(parse(layout(), &bad, &[1]).is_err());
    }
}
