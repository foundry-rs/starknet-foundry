use crate::command::{USCError, USCInternalCommand};
use crate::compile::{CompilationError, SierraType, compile_sierra_bytes};
use regex::Regex;
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock};
use tempfile::Builder;

pub const CASM_CACHE_DIR: &str = "casm";

// Bump when the cache key inputs or cached CASM JSON format/semantics change in a way that could
// make old entries deserialize successfully but no longer be valid for the current implementation.
const CACHE_SCHEMA_VERSION: &str = "v1";

static USC_VERSION: OnceLock<String> = OnceLock::new();
static PATH_SEGMENT_WHITESPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
static PATH_SEGMENT_UNSAFE_CHARS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^A-Za-z0-9-]+").unwrap());

pub fn compile_sierra_at_path_with_cache<T>(
    sierra_file_path: &Path,
    sierra_type: SierraType,
    cache_dir: &Path,
) -> Result<T, CompilationError>
where
    T: DeserializeOwned + Serialize,
{
    let sierra_bytes =
        fs::read(sierra_file_path).map_err(|source| CompilationError::SierraFileRead {
            path: sierra_file_path.to_path_buf(),
            source,
        })?;

    compile_sierra_bytes_with_cache(&sierra_bytes, sierra_type, cache_dir)
}

pub fn compile_sierra_bytes_with_cache<T>(
    sierra_bytes: &[u8],
    sierra_type: SierraType,
    cache_dir: &Path,
) -> Result<T, CompilationError>
where
    T: DeserializeOwned + Serialize,
{
    let compiler_version = compiler_version()?;

    compile_sierra_bytes_with_cache_using_version(
        sierra_bytes,
        sierra_type,
        cache_dir,
        compiler_version,
        |sierra_bytes| compile_sierra_bytes(sierra_bytes, sierra_type),
    )
}

#[cfg(test)]
fn compile_sierra_at_path_with_cache_using_version<T>(
    sierra_file_path: &Path,
    sierra_type: SierraType,
    cache_dir: &Path,
    compiler_version: &str,
    compile: impl FnOnce(&[u8]) -> Result<String, CompilationError>,
) -> Result<T, CompilationError>
where
    T: DeserializeOwned + Serialize,
{
    let sierra_bytes =
        fs::read(sierra_file_path).map_err(|source| CompilationError::SierraFileRead {
            path: sierra_file_path.to_path_buf(),
            source,
        })?;

    compile_sierra_bytes_with_cache_using_version(
        &sierra_bytes,
        sierra_type,
        cache_dir,
        compiler_version,
        compile,
    )
}

fn compile_sierra_bytes_with_cache_using_version<T>(
    sierra_bytes: &[u8],
    sierra_type: SierraType,
    cache_dir: &Path,
    compiler_version: &str,
    compile: impl FnOnce(&[u8]) -> Result<String, CompilationError>,
) -> Result<T, CompilationError>
where
    T: DeserializeOwned + Serialize,
{
    let cache_file_path = cache_file_path(cache_dir, sierra_type, compiler_version, sierra_bytes);

    if let Some(casm) = read_cache_entry(&cache_file_path) {
        return Ok(casm);
    }

    let json = compile(sierra_bytes)?;
    let casm = serde_json::from_str(&json).map_err(CompilationError::Deserialization)?;

    if let Err(error) = write_cache_entry(&cache_file_path, &casm) {
        tracing::debug!(
            path = %cache_file_path.display(),
            %error,
            "failed to write CASM cache entry"
        );
    }

    Ok(casm)
}

fn compiler_version() -> Result<&'static str, USCError> {
    if let Some(version) = USC_VERSION.get() {
        return Ok(version);
    }

    let output = USCInternalCommand::new()?.arg("--version").run()?;
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let _ = USC_VERSION.set(version);

    Ok(USC_VERSION
        .get()
        .expect("USC version should be initialized"))
}

fn cache_file_path(
    cache_dir: &Path,
    sierra_type: SierraType,
    compiler_version: &str,
    sierra_bytes: &[u8],
) -> PathBuf {
    cache_dir
        .join(CASM_CACHE_DIR)
        .join(sierra_type.to_string())
        .join(sanitize_path_segment(compiler_version))
        .join(format!(
            "{}.json",
            cache_key(sierra_type, compiler_version, sierra_bytes)
        ))
}

fn cache_key(sierra_type: SierraType, compiler_version: &str, sierra_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(CACHE_SCHEMA_VERSION.as_bytes());
    hasher.update([0]);
    hasher.update(sierra_type.to_string().as_bytes());
    hasher.update([0]);
    hasher.update(compiler_version.as_bytes());
    hasher.update([0]);
    hasher.update(sierra_bytes);

    hex_encode(hasher.finalize())
}

fn read_cache_entry<T>(path: &Path) -> Option<T>
where
    T: DeserializeOwned,
{
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return None,
        Err(error) => {
            tracing::debug!(
                path = %path.display(),
                %error,
                "failed to read CASM cache entry"
            );
            return None;
        }
    };

    match serde_json::from_reader(file) {
        Ok(casm) => Some(casm),
        Err(error) => {
            tracing::debug!(
                path = %path.display(),
                %error,
                "failed to deserialize CASM cache entry"
            );
            None
        }
    }
}

fn write_cache_entry<T>(path: &Path, casm: &T) -> io::Result<()>
where
    T: Serialize,
{
    let parent = path
        .parent()
        .expect("CASM cache path should always have a parent directory");
    fs::create_dir_all(parent)?;

    let mut temp_file = Builder::new()
        .prefix(".casm-cache-")
        .suffix(".json")
        .tempfile_in(parent)?;
    serde_json::to_writer(&mut temp_file, casm).map_err(io::Error::other)?;
    temp_file.flush()?;
    temp_file.persist(path).map_err(|error| error.error)?;

    Ok(())
}

fn sanitize_path_segment(value: &str) -> String {
    let sanitized = PATH_SEGMENT_WHITESPACE.replace_all(value, "-");
    let sanitized = PATH_SEGMENT_UNSAFE_CHARS.replace_all(&sanitized, "_");
    let sanitized = sanitized.trim_matches(['_', '-']).to_string();

    if sanitized.is_empty() {
        fallback_path_segment(value)
    } else {
        sanitized
    }
}

fn fallback_path_segment(value: &str) -> String {
    let digest = hex_encode(Sha256::digest(value.as_bytes()));
    format!("hash-{}", &digest[..12])
}

fn hex_encode(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .fold(String::new(), |mut output, byte| {
            write!(&mut output, "{byte:02x}").expect("writing to String should not fail");
            output
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::representation::{AssembledCairoProgram, RawCasmProgram};

    fn raw_casm(debug_info: Vec<(usize, usize)>) -> RawCasmProgram {
        RawCasmProgram {
            assembled_cairo_program: AssembledCairoProgram {
                bytecode: vec![],
                hints: vec![],
            },
            debug_info,
        }
    }

    fn raw_casm_json(debug_info: Vec<(usize, usize)>) -> String {
        serde_json::to_string(&raw_casm(debug_info)).unwrap()
    }

    #[test]
    fn reuses_cache_for_equal_sierra_bytes_at_different_paths() {
        let temp = tempfile::tempdir().unwrap();
        let sierra_a = temp.path().join("first.json");
        let nested = temp.path().join("nested");
        fs::create_dir(&nested).unwrap();
        let sierra_b = nested.join("second.json");
        fs::write(&sierra_a, br#"{"program":"same"}"#).unwrap();
        fs::write(&sierra_b, br#"{"program":"same"}"#).unwrap();

        let cache_dir = temp.path().join("cache");
        let first = compile_sierra_at_path_with_cache_using_version::<RawCasmProgram>(
            &sierra_a,
            SierraType::Raw,
            &cache_dir,
            "universal-sierra-compiler 1.2.3",
            |_| Ok(raw_casm_json(vec![(1, 2)])),
        )
        .unwrap();
        let second = compile_sierra_at_path_with_cache_using_version::<RawCasmProgram>(
            &sierra_b,
            SierraType::Raw,
            &cache_dir,
            "universal-sierra-compiler 1.2.3",
            |_| panic!("cache hit should not compile again"),
        )
        .unwrap();

        assert_eq!(first, raw_casm(vec![(1, 2)]));
        assert_eq!(second, raw_casm(vec![(1, 2)]));
    }

    #[test]
    fn separates_cache_entries_by_compiler_version() {
        let temp = tempfile::tempdir().unwrap();
        let sierra_path = temp.path().join("program.json");
        fs::write(&sierra_path, br#"{"program":"same"}"#).unwrap();

        let cache_dir = temp.path().join("cache");
        compile_sierra_at_path_with_cache_using_version::<RawCasmProgram>(
            &sierra_path,
            SierraType::Raw,
            &cache_dir,
            "universal-sierra-compiler 1.2.3",
            |_| Ok(raw_casm_json(vec![(1, 2)])),
        )
        .unwrap();
        let result = compile_sierra_at_path_with_cache_using_version::<RawCasmProgram>(
            &sierra_path,
            SierraType::Raw,
            &cache_dir,
            "universal-sierra-compiler 1.2.4",
            |_| Ok(raw_casm_json(vec![(3, 4)])),
        )
        .unwrap();

        assert_eq!(result, raw_casm(vec![(3, 4)]));
    }

    #[test]
    fn separates_cache_entries_for_versions_with_same_sanitized_path_segment() {
        let temp = tempfile::tempdir().unwrap();
        let cache_dir = temp.path().join("cache");
        let sierra_bytes = br#"{"program":"same"}"#;
        let version_a = "universal-sierra-compiler 1.2.3";
        let version_b = "universal-sierra-compiler-1/2/3";

        assert_eq!(
            sanitize_path_segment(version_a),
            sanitize_path_segment(version_b)
        );

        let cache_file_a = cache_file_path(&cache_dir, SierraType::Raw, version_a, sierra_bytes);
        let cache_file_b = cache_file_path(&cache_dir, SierraType::Raw, version_b, sierra_bytes);

        assert_eq!(cache_file_a.parent(), cache_file_b.parent());
        assert_ne!(cache_file_a.file_name(), cache_file_b.file_name());
    }

    #[test]
    fn recompiles_and_overwrites_corrupt_cache_entry() {
        let temp = tempfile::tempdir().unwrap();
        let sierra_path = temp.path().join("program.json");
        fs::write(&sierra_path, br#"{"program":"same"}"#).unwrap();

        let cache_dir = temp.path().join("cache");
        let sierra_bytes = fs::read(&sierra_path).unwrap();
        let cache_file = cache_file_path(
            &cache_dir,
            SierraType::Raw,
            "universal-sierra-compiler 1.2.3",
            &sierra_bytes,
        );
        fs::create_dir_all(cache_file.parent().unwrap()).unwrap();
        fs::write(&cache_file, "not valid json").unwrap();

        let result = compile_sierra_at_path_with_cache_using_version::<RawCasmProgram>(
            &sierra_path,
            SierraType::Raw,
            &cache_dir,
            "universal-sierra-compiler 1.2.3",
            |_| Ok(raw_casm_json(vec![(5, 6)])),
        )
        .unwrap();
        let cached: RawCasmProgram =
            serde_json::from_str(&fs::read_to_string(cache_file).unwrap()).unwrap();

        assert_eq!(result, raw_casm(vec![(5, 6)]));
        assert_eq!(cached, raw_casm(vec![(5, 6)]));
    }

    #[test]
    fn compiles_snapshot_bytes_when_source_path_changes() {
        let temp = tempfile::tempdir().unwrap();
        let sierra_path = temp.path().join("program.json");
        let old_sierra = br#"{"program":"old"}"#;
        let new_sierra = br#"{"program":"new"}"#;
        fs::write(&sierra_path, old_sierra).unwrap();

        let cache_dir = temp.path().join("cache");
        let result = compile_sierra_at_path_with_cache_using_version::<RawCasmProgram>(
            &sierra_path,
            SierraType::Raw,
            &cache_dir,
            "universal-sierra-compiler 1.2.3",
            |sierra_bytes| {
                fs::write(&sierra_path, new_sierra).unwrap();
                assert_eq!(sierra_bytes, old_sierra);
                Ok(raw_casm_json(vec![(7, 8)]))
            },
        )
        .unwrap();

        let old_cache_file = cache_file_path(
            &cache_dir,
            SierraType::Raw,
            "universal-sierra-compiler 1.2.3",
            old_sierra,
        );
        let new_cache_file = cache_file_path(
            &cache_dir,
            SierraType::Raw,
            "universal-sierra-compiler 1.2.3",
            new_sierra,
        );

        assert_eq!(result, raw_casm(vec![(7, 8)]));
        assert!(old_cache_file.exists());
        assert!(!new_cache_file.exists());
    }

    #[test]
    fn returns_compiled_result_when_cache_entry_cannot_be_written() {
        let temp = tempfile::tempdir().unwrap();
        let sierra_path = temp.path().join("program.json");
        fs::write(&sierra_path, br#"{"program":"same"}"#).unwrap();

        let cache_dir = temp.path().join("cache");
        fs::create_dir(&cache_dir).unwrap();
        fs::write(cache_dir.join(CASM_CACHE_DIR), "not a directory").unwrap();

        let result = compile_sierra_at_path_with_cache_using_version::<RawCasmProgram>(
            &sierra_path,
            SierraType::Raw,
            &cache_dir,
            "universal-sierra-compiler 1.2.3",
            |_| Ok(raw_casm_json(vec![(9, 10)])),
        )
        .unwrap();

        assert_eq!(result, raw_casm(vec![(9, 10)]));
        assert!(cache_dir.join(CASM_CACHE_DIR).is_file());
    }

    #[test]
    fn sanitizes_compiler_version_for_directory_name() {
        assert_eq!(
            sanitize_path_segment("universal-sierra-compiler 2.9.0"),
            "universal-sierra-compiler-2_9_0"
        );
        assert_eq!(sanitize_path_segment("../"), "hash-fa08499e14d0");
    }
}
