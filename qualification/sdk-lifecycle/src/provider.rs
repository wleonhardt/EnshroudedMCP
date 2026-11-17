//! Published immutable loader metadata only. No World, Input or command handles.
use enshwick::abi::Header;
#[cfg(not(test))]
use enshwick::{Load, Mod, Off};
use serde_json::{Value, json};
use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::Path,
};

const REPORT: &str = "provider-observation.json";

fn observation(header: Header) -> Value {
    json!({"compiled_sdk_revision":"6a5966ee6683fde480ac9dd3654f91e73da6e2a5",
        "compiled_abi":{"major":enshwick::abi::ABI_MAJOR,"minor":enshwick::abi::ABI_MINOR},
        "compiled_table_bytes":std::mem::size_of::<enshwick::abi::HostApi>(),
        "reported_provider_header":{"abi_major":header.abi_major,"abi_minor":header.abi_minor,"table_bytes":header.size},
        "source":"published_host_api_immutable_header",
        "world_read":false,"commands_submitted":false,"credential_store_read":false,
        "provider_signatures_verified":false,"native_lifecycle_verified":false,"rendered_verified":false})
}

fn write_once(dir: &Path, value: &Value) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join(REPORT))?;
    serde_json::to_writer_pretty(&mut file, value).map_err(io::Error::other)?;
    file.write_all(b"\n")?;
    file.sync_all()
}

#[cfg(not(test))]
struct ProviderProbe;
#[cfg(not(test))]
impl Mod for ProviderProbe {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        // This published process-lifetime ABI header is not a World observation.
        let value = observation(load.host().api().header);
        let dir = load.mod_dir()?;
        write_once(&dir, &value)
            .map_err(|_| Off::new("provider diagnostic unavailable; no retry"))?;
        Ok(Self)
    }
}

#[cfg(not(test))]
enshwick::declare_mod!(ProviderProbe);

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    #[test]
    fn reports_actual_header_fields_without_substituting_compiled_constants() {
        let copied = observation(Header {
            abi_major: 42,
            abi_minor: 99,
            size: 4096,
        });
        assert_eq!(
            copied["reported_provider_header"],
            json!({"abi_major":42,"abi_minor":99,"table_bytes":4096})
        );
        assert_eq!(copied["compiled_abi"], json!({"major":12,"minor":11}));
        assert_eq!(copied["world_read"], false);
        assert_eq!(copied["commands_submitted"], false);
        assert_eq!(copied["native_lifecycle_verified"], false);
    }

    #[test]
    fn exclusive_diagnostic_preserves_existing_bytes_and_missing_directory_refuses() {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let parent = root.join("gamedata/qualification-tests/lifecycle-provider");
        fs::create_dir_all(&parent).unwrap();
        let parent = parent.canonicalize().unwrap();
        assert!(parent.starts_with(&root));
        let dir = parent.join(format!(
            "{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        assert!(dir.canonicalize().unwrap().starts_with(parent));
        let value = observation(Header::of::<enshwick::abi::HostApi>());
        assert_eq!(
            write_once(&dir.join("missing"), &value).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        write_once(&dir, &value).unwrap();
        let bytes = fs::read(dir.join(REPORT)).unwrap();
        assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), value);
        assert_eq!(
            write_once(&dir, &json!({"replacement":true}))
                .unwrap_err()
                .kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(fs::read(dir.join(REPORT)).unwrap(), bytes);
        fs::remove_file(dir.join(REPORT)).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
