use std::{
    env,
    error::Error,
    fs,
    io::{self, Write},
    path::Path,
};

use flate2::{Compression, GzBuilder};
use sha2::{Digest, Sha256};

fn main() -> Result<(), Box<dyn Error>> {
    let seeds = Path::new(env!("CARGO_MANIFEST_DIR")).join("seeds");
    let output = env::var_os("OUT_DIR").ok_or_else(|| io::Error::other("OUT_DIR is missing"))?;
    let output = Path::new(&output);
    println!("cargo::rerun-if-changed={}", seeds.display());

    let mut files = fs::read_dir(&seeds)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    files.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    files.sort();

    let mut provinces = String::from("pub static FILES: &[EmbeddedSeed] = &[\n");
    let mut cities = provinces.clone();
    let mut polling_methods = provinces.clone();
    let mut indonesia_boundary = provinces.clone();
    let mut expected_version = 1;

    for path in files {
        let filename = path
            .file_name()
            .and_then(|filename| filename.to_str())
            .ok_or_else(|| io::Error::other("seed filename is not UTF-8"))?;
        let name = filename.trim_end_matches(".json");
        let (prefix, name) = name
            .split_once('_')
            .ok_or_else(|| io::Error::other(format!("invalid seed filename: {filename}")))?;
        let version: i64 = prefix.parse()?;
        let group = match version {
            1..=7 if name.starts_with("provinces_") => &mut provinces,
            8..=45 if name.starts_with("cities_") => &mut cities,
            46 if name == "polling_methods" => &mut polling_methods,
            47 if name == "indonesia_boundary" => &mut indonesia_boundary,
            _ => return Err(io::Error::other(format!("unsupported seed: {filename}")).into()),
        };
        if version != expected_version {
            return Err(io::Error::other(format!(
                "expected seed version {expected_version}, got {filename}"
            ))
            .into());
        }

        let raw = fs::read(&path)?;
        let metadata: serde_json::Value = serde_json::from_slice(&raw)?;
        if metadata["version"].as_i64() != Some(version)
            || metadata["name"].as_str() != Some(name)
            || (version != 46 && metadata["srid"].as_u64() != Some(4326))
        {
            return Err(io::Error::other(format!(
                "seed envelope does not match filename or SRID: {filename}"
            ))
            .into());
        }

        // Preserve exact source bytes for migration-style checksums and deterministic gzip output.
        let checksum = Sha256::digest(&raw)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let mut encoder = GzBuilder::new()
            .mtime(0)
            .operating_system(255)
            .write(Vec::new(), Compression::best());
        encoder.write_all(&raw)?;
        fs::write(output.join(format!("{filename}.gz")), encoder.finish()?)?;

        group.push_str(&format!(
            "    EmbeddedSeed {{ version: {version}, filename: {filename:?}, checksum_sha256: {checksum:?}, compressed: include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{filename}.gz\")) }},\n"
        ));
        expected_version += 1;
    }

    if expected_version != 48 {
        return Err(io::Error::other(
            "expected all 47 geographic, polling method, and boundary seed files",
        )
        .into());
    }
    provinces.push_str("];\n");
    cities.push_str("];\n");
    polling_methods.push_str("];\n");
    indonesia_boundary.push_str("];\n");
    fs::write(output.join("province_seeds.rs"), provinces)?;
    fs::write(output.join("city_seeds.rs"), cities)?;
    fs::write(output.join("polling_method_seeds.rs"), polling_methods)?;
    fs::write(
        output.join("indonesia_boundary_seeds.rs"),
        indonesia_boundary,
    )?;
    Ok(())
}
