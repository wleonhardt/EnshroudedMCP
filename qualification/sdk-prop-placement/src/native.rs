//! One diagnostic report after the published world-loaded event and scheduler tick.
use enshwick::{ItemId, Load, Mod, Off, events::WorldLoaded, inventory::Inventories, world::World};
use serde_json::{Value, json};
use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::Path,
};

const SDK: &str = "9f6efc821524df426991fb584a761eaa0e5ea898";
const REPORT: &str = "placement-observation.json";
// The known box selector is already part of the fixed product probe contract; zero is a refusal case.
const ITEMS: [ItemId; 2] = [ItemId::new(213939880), ItemId::new(0)];

fn sample(world: World, inventories: Option<Inventories>, item: ItemId) -> Value {
    let (metadata, error) = match crate::metadata::copy_metadata(world, item) {
        Ok(value) => (Some(value), None),
        Err(error) => (None, Some(format!("{error:?}"))),
    };
    let independent = inventories.and_then(|reader| reader.item_info(item));
    let (mapping_state, template) = match independent {
        Some(info) => match info.placed_entity_template {
            Some(template) => ("observed", Some(crate::metadata::template_hex(template))),
            None => ("known_no_template", None),
        },
        None => ("unavailable", None),
    };
    let agreement = metadata
        .as_ref()
        .zip(template.as_ref())
        .map(|(value, template)| {
            value["template_guid_file_order_hex"].as_str() == Some(template.as_str())
        });
    json!({"item_id":item.0,"metadata":metadata,"error":error,
        "independent_item_info":{"state":mapping_state,"template_guid_file_order_hex":template},
        "template_correspondence_observed":agreement})
}

fn write_once(dir: &Path, world: World, inventories: Option<Inventories>) -> io::Result<()> {
    // Claim the diagnostic before any metadata query; never overwrite or retry a partial report.
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join(REPORT))?;
    let value = json!({"compiled_sdk_revision":SDK,
        "compiled_abi":format!("{}.{}",enshwick::abi::ABI_MAJOR,enshwick::abi::ABI_MINOR),
        "samples":ITEMS.map(|item|sample(world,inventories,item)),
        "read_only_intent":true,"runtime_provider_verified":false,
        "world_preservation_verified":false,"rendered_verified":false});
    serde_json::to_writer_pretty(&mut file, &value).map_err(io::Error::other)?;
    file.write_all(b"\n")?;
    file.sync_all()
}

struct Probe;
impl Mod for Probe {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        let world = load.world()?;
        let scheduler = load.scheduler()?;
        let inventories = load.inventories().ok();
        let dir = load.mod_dir()?;
        load.on::<WorldLoaded>(move |()| {
            let dir = dir.clone();
            scheduler.next_tick(move || {
                if let Err(error) = write_once(&dir, world, inventories) {
                    enshwick::warn!("placement qualification report unavailable: {error}");
                }
            });
        })?;
        Ok(Self)
    }
}

#[cfg(not(test))]
enshwick::declare_mod!(Probe);

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::{Entity, abi::Status, content::Guid, events::WorldLoaded};
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::Duration,
    };

    struct Fixture(PathBuf, PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap();
            let parent = root.join("target/sdk128-native-tests");
            fs::create_dir_all(&parent).unwrap();
            let parent = parent.canonicalize().unwrap();
            assert!(parent.starts_with(root));
            let dir = parent.join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&dir).unwrap();
            Self(dir, parent)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let dir = self.0.canonicalize().unwrap();
            assert_eq!(dir.parent(), Some(self.1.as_path()));
            fs::remove_dir_all(dir).unwrap();
        }
    }
    enshwick::test::declare_mod!(
        Probe,
        "id = \"test.native-placement-probe\"\nside = \"server\""
    );

    fn start(dir: &Fixture, with_inventory: bool) -> enshwick::test::Installed {
        let fake = enshwick::test::FakeHost::new()
            .world()
            .without_entity_commands()
            .mod_dir(dir.0.to_str().unwrap())
            .item(ITEMS[0], 1, 0)
            .item_places(ITEMS[0], Guid([17; 16]))
            .placement_box(ITEMS[0], [-1., 0., -0.5], [1., 2., 0.5])
            .prop(Entity(42), ITEMS[0], [100.; 3]);
        let host = if with_inventory {
            fake.inventory_requests()
        } else {
            fake
        }
        .install();
        // SAFETY: these are this module's framework-generated exports.
        assert_eq!(
            unsafe { host.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        host
    }

    #[test]
    fn event_defers_bounded_report_and_preserves_world_and_unknowns() {
        let dir = Fixture::new();
        let host = start(&dir, true);
        let before = host.props();
        let path = dir.0.join(REPORT);
        assert!(!path.exists());
        assert_eq!(host.raise::<WorldLoaded>(&()), 1);
        assert!(
            !path.exists(),
            "event must queue metadata reads rather than execute them"
        );
        let _ = host.tick(Duration::from_millis(100));
        let value: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(value["compiled_sdk_revision"], SDK);
        assert_eq!(value["compiled_abi"], "12.8");
        assert_eq!(value["samples"].as_array().unwrap().len(), 2);
        assert_eq!(
            value["samples"][0]["metadata"]["template_guid_file_order_hex"],
            "11".repeat(16)
        );
        assert_eq!(
            value["samples"][0]["metadata"]["declared_local_box_m"],
            serde_json::json!({"min":[-1.,0.,-0.5],"max":[1.,2.,0.5]})
        );
        assert_eq!(
            value["samples"][0]["template_correspondence_observed"],
            true
        );
        assert_eq!(value["samples"][1]["metadata"], serde_json::Value::Null);
        assert_eq!(value["samples"][1]["error"], "Refused");
        assert_eq!(
            value["samples"][1]["template_correspondence_observed"],
            serde_json::Value::Null
        );
        assert_eq!(value["runtime_provider_verified"], false);
        assert_eq!(value["world_preservation_verified"], false);
        assert_eq!(host.props(), before);
        assert!(host.transactions().is_empty());
    }

    #[test]
    fn missing_independent_reader_is_unknown_not_template_disagreement() {
        let dir = Fixture::new();
        let host = start(&dir, false);
        let _ = host.raise::<WorldLoaded>(&());
        let _ = host.tick(Duration::from_millis(100));
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(dir.0.join(REPORT)).unwrap()).unwrap();
        assert!(value["samples"][0]["metadata"].is_object());
        assert_eq!(
            value["samples"][0]["independent_item_info"]["state"],
            "unavailable"
        );
        assert!(value["samples"][0]["template_correspondence_observed"].is_null());
        assert!(host.transactions().is_empty());
    }

    #[test]
    fn later_world_event_never_overwrites_first_report() {
        let dir = Fixture::new();
        let host = start(&dir, true);
        let _ = host.raise::<WorldLoaded>(&());
        let _ = host.tick(Duration::from_millis(100));
        let path = dir.0.join(REPORT);
        let mut first = fs::read(&path).unwrap();
        first.extend_from_slice(b"operator annotation");
        fs::write(&path, &first).unwrap();
        assert_eq!(host.raise::<WorldLoaded>(&()), 1);
        assert!(host.tick(Duration::from_millis(100)) > 0);
        assert!(host.logs().iter().any(|(_, message)| {
            message.starts_with("placement qualification report unavailable:")
        }));
        assert_eq!(fs::read(path).unwrap(), first);
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
        assert!(host.transactions().is_empty());
    }

    #[test]
    fn preexisting_diagnostic_refuses_without_replacing_owned_bytes() {
        let dir = Fixture::new();
        let host = start(&dir, true);
        let path = dir.0.join(REPORT);
        fs::write(&path, b"existing observation").unwrap();
        let _ = host.raise::<WorldLoaded>(&());
        let _ = host.tick(Duration::from_millis(100));
        assert_eq!(fs::read(path).unwrap(), b"existing observation");
        assert!(host.logs().iter().any(|(_, message)| {
            message.starts_with("placement qualification report unavailable:")
        }));
        assert!(host.transactions().is_empty());
    }
}
