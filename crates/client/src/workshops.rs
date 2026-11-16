//! Published vanilla workshop-to-station definitions, not station availability.
use enshwick::{
    Host,
    content::{
        Blob, Guid, types,
        workshops::{WORKSHOP_REGISTRY, WorkshopLayout},
    },
};
use policy::{client_recipe::bool_byte, client_workshop::Layout};
use serde_json::{Value, json};

pub(crate) fn read(
    host: Host,
    layout: Option<WorkshopLayout>,
    id: u32,
    limit: usize,
) -> Result<Value, String> {
    let l = layout.ok_or("workshop layout unavailable")?;
    let key = types::WORKSHOP_REGISTRY_RESOURCE
        .key(WORKSHOP_REGISTRY)
        .map_err(|e| e.to_string())?;
    let bytes = host
        .game_resource(key)
        .ok_or("vanilla workshop registry unavailable")?;
    parse(
        Layout {
            array: l.workshops,
            stride: l.entry,
            guid: l.entry_guid,
            id: l.entry_id,
            name: l.entry_name,
            item: l.entry_item,
            item_ref: l.entry_item_ref,
            npc: l.entry_npc,
        },
        &bytes,
        id,
        limit,
    )
}

fn parse(layout: Layout, bytes: &[u8], id: u32, limit: usize) -> Result<Value, String> {
    layout.check(bytes.len(), limit)?;
    let blob = Blob::new(bytes);
    let (start, count) = blob
        .array(layout.array)
        .ok_or("malformed workshop array header")?;
    let count = count as usize;
    layout.entries(start, count, bytes.len())?;
    let mut matches = Vec::new();
    let mut total = 0;
    for index in 0..count {
        let at = start + index * layout.stride;
        let u32_at = |field| {
            blob.read_u32(at + field)
                .ok_or("malformed workshop u32 field")
        };
        if u32_at(layout.id)? != id {
            continue;
        }
        total += 1;
        if matches.len() >= limit {
            continue;
        }
        let guid = |field| {
            Guid(
                bytes[at + field..at + field + 16]
                    .try_into()
                    .expect("admitted GUID field"),
            )
            .to_string()
        };
        matches.push(json!({"workshop_id":id,"workshop_guid":guid(layout.guid),"name_tag":u32_at(layout.name)?,
            "station_item_id":u32_at(layout.item)?,"station_item_guid":guid(layout.item_ref),"npc_workshop":bool_byte(bytes[at+layout.npc])?}));
    }
    Ok(
        json!({"mode":"report_only","source":"vanilla_archive_workshop_registry","identity_scope":"recipe_workshop_definition",
        "effective_definition_verified":false,"current_station_available":null,"current_availability_verified":false,
        "workshop_id":id,"registry_entries":count,"total_matches":total,"truncated":total>matches.len(),
        "state":match total {0=>"not_found",1=>"single_match",_=>"multiple_matches"},"matches":matches,
        "limitations":["vanilla definitions do not establish current stations, unlocks or crafting eligibility","workshop definition IDs are not item or entity selectors","whole upstream resource allocation precedes parser byte admission"]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn layout() -> Layout {
        Layout {
            array: 0,
            stride: 56,
            guid: 0,
            id: 16,
            name: 20,
            item: 24,
            item_ref: 28,
            npc: 44,
        }
    }
    fn fixture() -> Vec<u8> {
        let mut b = Blob::new(&[0; 8]);
        let start = b.append(&[0; 168], 4);
        assert!(b.set_array(0, start, 3));
        for (index, id) in [0, u32::MAX, u32::MAX].into_iter().enumerate() {
            let at = start + index * 56;
            assert!(b.write(at, &[[3u8, 4, 5][index]; 16]));
            assert!(b.write(at + 16, &id.to_le_bytes()));
            assert!(b.write(at + 20, &73u32.to_le_bytes()));
            assert!(b.write(at + 24, &991u32.to_le_bytes()));
            assert!(b.write(at + 28, &[5; 16]));
            assert!(b.write(at + 44, &[1]));
        }
        b.into_bytes()
    }
    #[test]
    fn raw_id_collisions_zero_and_truncation_preserve_declared_station_identity() {
        let bytes = fixture();
        let one = parse(layout(), &bytes, 0, 8).unwrap();
        assert_eq!(one["state"], "single_match");
        assert_eq!(one["matches"][0]["station_item_id"], 991);
        assert_eq!(
            one["matches"][0]["station_item_guid"],
            Guid([5; 16]).to_string()
        );
        assert_eq!(one["matches"][0]["npc_workshop"], true);
        assert_eq!(one["current_station_available"], Value::Null);
        assert_eq!(one["current_availability_verified"], false);
        let many = parse(layout(), &bytes, u32::MAX, 1).unwrap();
        assert_eq!(many["state"], "multiple_matches");
        assert_eq!(many["total_matches"], 2);
        assert_eq!(many["matches"].as_array().unwrap().len(), 1);
        assert_eq!(many["truncated"], true);
        let full = parse(layout(), &bytes, u32::MAX, 8).unwrap();
        assert_eq!(full["matches"].as_array().unwrap().len(), 2);
        assert_eq!(full["truncated"], false);
        assert_eq!(
            full["matches"][0]["workshop_guid"],
            Guid([4; 16]).to_string()
        );
        assert_eq!(
            full["matches"][1]["workshop_guid"],
            Guid([5; 16]).to_string()
        );
        let missing = parse(layout(), &bytes, 73, 8).unwrap();
        assert_eq!(missing["state"], "not_found");
        assert_eq!(missing["matches"], json!([]));
    }
    #[test]
    fn malformed_records_headers_boolean_and_workloads_refuse() {
        let bytes = fixture();
        assert!(parse(layout(), &bytes[..bytes.len() - 1], 0, 8).is_err());
        let mut b = Blob::new(&bytes);
        let (start, _) = b.array(0).unwrap();
        assert!(b.write(start + 44, &[2]));
        assert!(parse(layout(), b.bytes(), 0, 8).is_err());
        assert!(parse(layout(), &bytes, 0, 0).is_err());
        assert!(
            parse(
                Layout {
                    guid: 41,
                    ..layout()
                },
                &bytes,
                0,
                8
            )
            .is_err()
        );
        let mut b = Blob::new(&bytes);
        assert!(b.set_array(0, start, 4097));
        assert!(parse(layout(), b.bytes(), 0, 8).is_err());
        assert!(parse(layout(), &vec![0; policy::item_lookup::MAX_BYTES + 1], 0, 8).is_err());
    }
}
