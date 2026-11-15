//! Bounded observations of props; labels are presence only, never recovery authority.

use enshwick::Entity;
use enshwick::world::Prop;
use serde_json::{Value, json};

pub(crate) fn rows(
    props: &[Prop],
    mut inspect: impl FnMut(Entity) -> (Option<[u8; 16]>, Option<bool>),
) -> Vec<Value> {
    props.iter().take(policy::inspection::PROP_LIMIT).map(|prop| {
        let (template, label) = inspect(prop.entity);
        let template = template.map(|bytes| bytes.iter().map(|byte|format!("{byte:02x}")).collect::<String>());
        json!({"entity":prop.entity.0,"item":prop.item.0,"position":prop.transform.position,
            "orientation":prop.transform.orientation,"template_guid":template,"label_present":label})
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::world::{Prop, WorldTransform};
    use enshwick::{Entity, ItemId};
    use serde_json::json;

    fn prop(entity: u32) -> Prop {
        Prop {
            entity: Entity(entity),
            item: ItemId::new(7),
            transform: WorldTransform::at([1., 2., 3.]),
        }
    }

    #[test]
    fn observed_false_and_unavailable_labels_are_distinct() {
        let props = [prop(1), prop(2), prop(3)];
        let out = rows(&props, |entity| match entity.0 {
            1 => (Some(std::array::from_fn(|i| i as u8)), Some(false)),
            2 => (None, None),
            _ => (Some([2; 16]), Some(true)),
        });
        assert_eq!(out[0]["label_present"], false);
        assert!(out[1]["label_present"].is_null());
        assert_eq!(out[2]["label_present"], true);
        assert_eq!(out[0]["template_guid"], "000102030405060708090a0b0c0d0e0f");
        assert!(out[1]["template_guid"].is_null());
        assert_eq!(out[0]["entity"], 1);
        assert_eq!(out[0]["item"], 7);
        assert_eq!(out[0]["position"], json!([1., 2., 3.]));
        assert_eq!(out[0]["orientation"], json!([0., 0., 0., 1.]));
    }

    #[test]
    fn metadata_is_read_only_for_returned_rows() {
        let props: Vec<_> = (1..=501).map(prop).collect();
        let mut inspected = Vec::new();
        let out = rows(&props, |entity| {
            inspected.push(entity.0);
            (None, None)
        });
        assert_eq!(out.len(), 500);
        assert_eq!(inspected, (1..=500).collect::<Vec<_>>());
    }
}
