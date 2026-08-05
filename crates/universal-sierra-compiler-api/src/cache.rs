use crate::command::{USCError, USCInternalCommand};
use crate::compile::{CompilationError, SierraType, compile_sierra_at_path, compile_sierra_bytes};
use cairo_lang_sierra::program::Program;
use regex::Regex;
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::io::{self, BufReader, BufWriter, Write as _};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, OnceLock};
use tempfile::Builder;

pub const CASM_CACHE_DIR: &str = "casm";

// snforge's release version, keyed into every cache entry so upgrading snforge starts from a fresh
// cache namespace (the same approach as the fork cache's `cache_version`). This crate is
// workspace-versioned, so `CARGO_PKG_VERSION` bumps on every release; any change that could alter the
// cached CASM ships in a release and therefore moves this version: a `cairo-lang-*` bump that changes
// the serialized representation (`CasmContractClass` / the `cairo-lang-casm` types in
// `RawCasmProgram`), or an in-repo change to `RawCasmProgram` itself. Sierra -> CASM codegen changes
// come from the separately installed USC binary and are covered by its version in `cache_key`.
const SNFORGE_VERSION: &str = env!("CARGO_PKG_VERSION");

static USC_VERSION: OnceLock<String> = OnceLock::new();
static USC_VERSION_LOCK: Mutex<()> = Mutex::new(());
static PATH_SEGMENT_WHITESPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
static PATH_SEGMENT_UNSAFE_CHARS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^A-Za-z0-9-]+").unwrap());

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SierraProgramHash(String);

// `debug_info` is not an input to Sierra -> CASM codegen, so it must not invalidate the raw CASM
// cache. See the Cairo Sierra -> CASM compiler API:
// https://github.com/starkware-libs/cairo/blob/eea264fa54fac04a1a5745ad533a0c0ab3106ab3/crates/cairo-lang-sierra-to-casm/src/compiler.rs#L441
#[must_use]
pub fn raw_sierra_program_content_hash(sierra_program: &Program) -> SierraProgramHash {
    let mut hasher = Sha256::new();
    write_serializable_to_hash(&mut hasher, sierra_program);
    SierraProgramHash(hex_encode(hasher.finalize()))
}

#[must_use]
pub fn contract_sierra_content_hash(sierra_bytes: &[u8]) -> SierraProgramHash {
    let mut hasher = Sha256::new();
    hasher.update(sierra_bytes);
    SierraProgramHash(hex_encode(hasher.finalize()))
}

pub(crate) fn compile_sierra_at_path_with_content_hash<T>(
    sierra_file_path: &Path,
    sierra_type: SierraType,
    cache_dir: &Path,
    sierra_content_hash: &SierraProgramHash,
) -> Result<T, CompilationError>
where
    T: DeserializeOwned + Serialize,
{
    compile_sierra_with_content_hash_using_version(
        sierra_type,
        cache_dir,
        compiler_version()?,
        sierra_content_hash,
        || compile_sierra_at_path(sierra_file_path, sierra_type),
    )
}

pub(crate) fn compile_sierra_bytes_with_content_hash<T>(
    sierra_bytes: &[u8],
    sierra_type: SierraType,
    cache_dir: &Path,
    sierra_content_hash: &SierraProgramHash,
) -> Result<T, CompilationError>
where
    T: DeserializeOwned + Serialize,
{
    let compiler_version = compiler_version()?;

    compile_sierra_with_content_hash_using_version(
        sierra_type,
        cache_dir,
        compiler_version,
        sierra_content_hash,
        || compile_sierra_bytes(sierra_bytes, sierra_type),
    )
}

fn compile_sierra_with_content_hash_using_version<T>(
    sierra_type: SierraType,
    cache_dir: &Path,
    compiler_version: &str,
    sierra_content_hash: &SierraProgramHash,
    compile: impl FnOnce() -> Result<String, CompilationError>,
) -> Result<T, CompilationError>
where
    T: DeserializeOwned + Serialize,
{
    let cache_file_path = cache_file_path_for_content_hash(
        cache_dir,
        sierra_type,
        compiler_version,
        sierra_content_hash,
    );

    if let Some(cache_file_path) = &cache_file_path
        && let Some(casm) = read_cache_entry(cache_file_path)
    {
        return Ok(casm);
    }

    let json = compile()?;
    let casm = serde_json::from_str(&json).map_err(CompilationError::Deserialization)?;

    match &cache_file_path {
        Some(cache_file_path) => {
            if let Err(error) = write_cache_entry(cache_file_path, &casm) {
                tracing::debug!(
                    path = %cache_file_path.display(),
                    %error,
                    "failed to write CASM cache entry"
                );
            }
        }
        None => {
            tracing::debug!(
                compiler_version,
                "skipping CASM cache: compiler version has no valid path segment"
            );
        }
    }

    Ok(casm)
}

fn compiler_version() -> Result<&'static str, USCError> {
    if let Some(version) = USC_VERSION.get() {
        return Ok(version);
    }

    let _guard = USC_VERSION_LOCK
        .lock()
        .expect("USC version lock should not be poisoned");
    if let Some(version) = USC_VERSION.get() {
        return Ok(version);
    }

    let output = USCInternalCommand::new()?.arg("--version").run()?;
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(USC_VERSION.get_or_init(|| version))
}

fn cache_file_path_for_content_hash(
    cache_dir: &Path,
    sierra_type: SierraType,
    compiler_version: &str,
    sierra_content_hash: &SierraProgramHash,
) -> Option<PathBuf> {
    Some(
        cache_dir
            .join(CASM_CACHE_DIR)
            .join(sierra_type.to_string())
            .join(sanitize_path_segment(SNFORGE_VERSION)?)
            .join(sanitize_path_segment(compiler_version)?)
            .join(format!(
                "{}.json",
                cache_key(sierra_type, compiler_version, sierra_content_hash)
            )),
    )
}

fn cache_key(
    sierra_type: SierraType,
    compiler_version: &str,
    sierra_content_hash: &SierraProgramHash,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(SNFORGE_VERSION.as_bytes());
    hasher.update([0]);
    hasher.update(sierra_type.to_string().as_bytes());
    hasher.update([0]);
    hasher.update(compiler_version.as_bytes());
    hasher.update([0]);
    hasher.update(sierra_content_hash.0.as_bytes());

    hex_encode(hasher.finalize())
}

fn write_serializable_to_hash(hasher: &mut Sha256, value: &impl Serialize) {
    serde_json::to_writer(HashWriter(hasher), value)
        .expect("writing JSON to a hasher should not fail");
}

struct HashWriter<'a>(&'a mut Sha256);

impl io::Write for HashWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[tracing::instrument(skip_all, level = "debug")]
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

    match serde_json::from_reader(BufReader::new(file)) {
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

#[tracing::instrument(skip_all, level = "debug")]
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

    {
        let mut writer = BufWriter::new(&mut temp_file);
        serde_json::to_writer(&mut writer, casm).map_err(io::Error::other)?;
        writer.flush()?;
    }

    temp_file.flush()?;
    temp_file.persist(path).map_err(|error| error.error)?;

    Ok(())
}

/// Turns a version string into a human-readable, filesystem-safe directory segment.
///
/// Returns `None` when the value sanitizes to an empty string (e.g. it consists only of
/// separators or whitespace). The caller then skips caching instead of inventing a name.
fn sanitize_path_segment(value: &str) -> Option<String> {
    let sanitized = PATH_SEGMENT_WHITESPACE.replace_all(value, "-");
    let sanitized = PATH_SEGMENT_UNSAFE_CHARS.replace_all(&sanitized, "_");
    let sanitized = sanitized.trim_matches(['_', '-']).to_string();

    if sanitized.is_empty() {
        None
    } else {
        Some(sanitized)
    }
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

    fn empty_sierra_program() -> Program {
        Program {
            type_declarations: vec![],
            libfunc_declarations: vec![],
            statements: vec![],
            funcs: vec![],
        }
    }

    fn sierra_program_with_return() -> Program {
        Program {
            statements: vec![cairo_lang_sierra::program::Statement::Return(vec![])],
            ..empty_sierra_program()
        }
    }

    #[test]
    fn raw_content_hash_ignores_sierra_json_field_order() {
        let program_a: Program = serde_json::from_str(
            r#"{"type_declarations":[],"libfunc_declarations":[],"statements":[],"funcs":[]}"#,
        )
        .unwrap();
        let program_b: Program = serde_json::from_str(
            r#"{"funcs":[],"statements":[],"libfunc_declarations":[],"type_declarations":[]}"#,
        )
        .unwrap();

        assert_eq!(
            raw_sierra_program_content_hash(&program_a),
            raw_sierra_program_content_hash(&program_b),
        );
    }

    // Raw hashing is computed over the typed `Program`, which carries no debug info, so builds that
    // differ only in debug info hash the same and reuse this entry.
    #[test]
    fn reuses_raw_cache_on_matching_content_hash() {
        let temp = tempfile::tempdir().unwrap();
        let cache_dir = temp.path().join("cache");
        let content_hash = raw_sierra_program_content_hash(&empty_sierra_program());
        let first = compile_sierra_with_content_hash_using_version::<RawCasmProgram>(
            SierraType::Raw,
            &cache_dir,
            "universal-sierra-compiler 1.2.3",
            &content_hash,
            || Ok(raw_casm_json(vec![(1, 2)])),
        )
        .unwrap();
        let second = compile_sierra_with_content_hash_using_version::<RawCasmProgram>(
            SierraType::Raw,
            &cache_dir,
            "universal-sierra-compiler 1.2.3",
            &content_hash,
            || panic!("a matching content hash should reuse the cached CASM entry"),
        )
        .unwrap();

        assert_eq!(first, raw_casm(vec![(1, 2)]));
        assert_eq!(second, raw_casm(vec![(1, 2)]));
    }

    #[test]
    fn reuses_contract_cache_with_precomputed_content_hash() {
        let temp = tempfile::tempdir().unwrap();
        let cache_dir = temp.path().join("cache");
        let sierra_bytes = br#"{"contract":"same"}"#;
        let content_hash = contract_sierra_content_hash(sierra_bytes);

        let first = compile_sierra_with_content_hash_using_version::<RawCasmProgram>(
            SierraType::Contract,
            &cache_dir,
            "universal-sierra-compiler 1.2.3",
            &content_hash,
            || Ok(raw_casm_json(vec![(1, 2)])),
        )
        .unwrap();

        let second = compile_sierra_with_content_hash_using_version::<RawCasmProgram>(
            SierraType::Contract,
            &cache_dir,
            "universal-sierra-compiler 1.2.3",
            &content_hash,
            || panic!("same contract Sierra content hash should reuse cache"),
        )
        .unwrap();

        assert_eq!(first, raw_casm(vec![(1, 2)]));
        assert_eq!(second, raw_casm(vec![(1, 2)]));
    }

    #[test]
    fn separates_cache_entries_by_compiler_version() {
        let temp = tempfile::tempdir().unwrap();
        let cache_dir = temp.path().join("cache");
        let content_hash = raw_sierra_program_content_hash(&empty_sierra_program());

        compile_sierra_with_content_hash_using_version::<RawCasmProgram>(
            SierraType::Raw,
            &cache_dir,
            "universal-sierra-compiler 1.2.3",
            &content_hash,
            || Ok(raw_casm_json(vec![(1, 2)])),
        )
        .unwrap();
        // Same program, newer USC => different key => must compile again.
        let result = compile_sierra_with_content_hash_using_version::<RawCasmProgram>(
            SierraType::Raw,
            &cache_dir,
            "universal-sierra-compiler 1.2.4",
            &content_hash,
            || Ok(raw_casm_json(vec![(3, 4)])),
        )
        .unwrap();

        assert_eq!(result, raw_casm(vec![(3, 4)]));
    }

    #[test]
    fn separates_cache_entries_for_versions_with_same_sanitized_path_segment() {
        let temp = tempfile::tempdir().unwrap();
        let cache_dir = temp.path().join("cache");
        let version_a = "universal-sierra-compiler 1.2.3";
        let version_b = "universal-sierra-compiler-1/2/3";

        assert_eq!(
            sanitize_path_segment(version_a),
            sanitize_path_segment(version_b)
        );

        let content_hash = raw_sierra_program_content_hash(&empty_sierra_program());
        let cache_file_a =
            cache_file_path_for_content_hash(&cache_dir, SierraType::Raw, version_a, &content_hash)
                .unwrap();
        let cache_file_b =
            cache_file_path_for_content_hash(&cache_dir, SierraType::Raw, version_b, &content_hash)
                .unwrap();

        assert_eq!(cache_file_a.parent(), cache_file_b.parent());
        assert_ne!(cache_file_a.file_name(), cache_file_b.file_name());
    }

    #[test]
    fn separates_cache_entries_by_sierra_type() {
        let temp = tempfile::tempdir().unwrap();
        let cache_dir = temp.path().join("cache");
        let version = "universal-sierra-compiler 1.2.3";
        let content_hash = raw_sierra_program_content_hash(&empty_sierra_program());

        // The same content hash under different Sierra types must never collide onto one entry.
        let raw =
            cache_file_path_for_content_hash(&cache_dir, SierraType::Raw, version, &content_hash)
                .unwrap();
        let contract = cache_file_path_for_content_hash(
            &cache_dir,
            SierraType::Contract,
            version,
            &content_hash,
        )
        .unwrap();

        assert_ne!(raw, contract);
    }

    #[test]
    fn skips_cache_when_compiler_version_has_no_valid_path_segment() {
        let temp = tempfile::tempdir().unwrap();
        let cache_dir = temp.path().join("cache");
        let content_hash = raw_sierra_program_content_hash(&empty_sierra_program());

        // A version string that sanitizes to empty cannot form a cache path, so the compiler runs
        // and nothing is cached...
        let first = compile_sierra_with_content_hash_using_version::<RawCasmProgram>(
            SierraType::Raw,
            &cache_dir,
            "../",
            &content_hash,
            || Ok(raw_casm_json(vec![(1, 2)])),
        )
        .unwrap();
        // ...so a second call must compile again instead of reusing an entry.
        let second = compile_sierra_with_content_hash_using_version::<RawCasmProgram>(
            SierraType::Raw,
            &cache_dir,
            "../",
            &content_hash,
            || Ok(raw_casm_json(vec![(3, 4)])),
        )
        .unwrap();

        assert_eq!(first, raw_casm(vec![(1, 2)]));
        assert_eq!(second, raw_casm(vec![(3, 4)]));
        assert!(!cache_dir.join(CASM_CACHE_DIR).exists());
    }

    #[test]
    fn recompiles_and_overwrites_corrupt_cache_entry() {
        let temp = tempfile::tempdir().unwrap();
        let cache_dir = temp.path().join("cache");
        let version = "universal-sierra-compiler 1.2.3";
        let content_hash = raw_sierra_program_content_hash(&empty_sierra_program());

        let cache_file =
            cache_file_path_for_content_hash(&cache_dir, SierraType::Raw, version, &content_hash)
                .unwrap();
        fs::create_dir_all(cache_file.parent().unwrap()).unwrap();
        fs::write(&cache_file, "not valid json").unwrap();

        let result = compile_sierra_with_content_hash_using_version::<RawCasmProgram>(
            SierraType::Raw,
            &cache_dir,
            version,
            &content_hash,
            || Ok(raw_casm_json(vec![(5, 6)])),
        )
        .unwrap();
        let cached: RawCasmProgram =
            serde_json::from_str(&fs::read_to_string(cache_file).unwrap()).unwrap();

        assert_eq!(result, raw_casm(vec![(5, 6)]));
        assert_eq!(cached, raw_casm(vec![(5, 6)]));
    }

    #[test]
    fn returns_compiled_result_when_cache_entry_cannot_be_written() {
        let temp = tempfile::tempdir().unwrap();
        let cache_dir = temp.path().join("cache");
        fs::create_dir(&cache_dir).unwrap();
        fs::write(cache_dir.join(CASM_CACHE_DIR), "not a directory").unwrap();

        let result = compile_sierra_with_content_hash_using_version::<RawCasmProgram>(
            SierraType::Raw,
            &cache_dir,
            "universal-sierra-compiler 1.2.3",
            &raw_sierra_program_content_hash(&empty_sierra_program()),
            || Ok(raw_casm_json(vec![(9, 10)])),
        )
        .unwrap();

        assert_eq!(result, raw_casm(vec![(9, 10)]));
        assert!(cache_dir.join(CASM_CACHE_DIR).is_file());
    }

    #[test]
    fn sanitizes_compiler_version_for_directory_name() {
        assert_eq!(
            sanitize_path_segment("universal-sierra-compiler 2.9.0").as_deref(),
            Some("universal-sierra-compiler-2_9_0")
        );
        // A version made only of separators sanitizes to nothing, so it has no valid path segment.
        assert_eq!(sanitize_path_segment("../"), None);
    }

    #[test]
    fn raw_program_content_hash_is_stable_and_content_sensitive() {
        // Same program -> same hash, so an unchanged program reuses its entry across runs.
        assert_eq!(
            raw_sierra_program_content_hash(&empty_sierra_program()),
            raw_sierra_program_content_hash(&empty_sierra_program()),
        );
        // Different program -> different hash, so a real code change gets a fresh entry.
        assert_ne!(
            raw_sierra_program_content_hash(&empty_sierra_program()),
            raw_sierra_program_content_hash(&sierra_program_with_return()),
        );
    }

    #[test]
    fn different_raw_programs_do_not_share_cache_entry() {
        let temp = tempfile::tempdir().unwrap();
        let cache_dir = temp.path().join("cache");
        let version = "universal-sierra-compiler 1.2.3";

        let first = compile_sierra_with_content_hash_using_version::<RawCasmProgram>(
            SierraType::Raw,
            &cache_dir,
            version,
            &raw_sierra_program_content_hash(&empty_sierra_program()),
            || Ok(raw_casm_json(vec![(1, 2)])),
        )
        .unwrap();
        // A different program hashes differently, so this must compile instead of hitting the entry
        // written above.
        let second = compile_sierra_with_content_hash_using_version::<RawCasmProgram>(
            SierraType::Raw,
            &cache_dir,
            version,
            &raw_sierra_program_content_hash(&sierra_program_with_return()),
            || Ok(raw_casm_json(vec![(3, 4)])),
        )
        .unwrap();

        assert_eq!(first, raw_casm(vec![(1, 2)]));
        assert_eq!(second, raw_casm(vec![(3, 4)]));
    }

    // Same for the production contract cache key, which comes from `contract_sierra_content_hash`
    // (called in `scarb-api`).
    #[test]
    fn contract_content_hash_is_stable_and_byte_sensitive() {
        let bytes = br#"{"sierra_program":["0x1"],"abi":"a"}"#;
        // Same bytes -> same hash.
        assert_eq!(
            contract_sierra_content_hash(bytes),
            contract_sierra_content_hash(bytes),
        );
        // Any change to the contract artifact -> different hash.
        assert_ne!(
            contract_sierra_content_hash(br#"{"sierra_program":["0x1"]}"#),
            contract_sierra_content_hash(br#"{"sierra_program":["0x2"]}"#),
        );
    }

    #[test]
    fn contract_content_hash_covers_debug_info_unlike_raw() {
        // Contracts are hashed over the whole artifact bytes (not the typed program), so even a
        // difference confined to debug info - which does not affect codegen - yields a different key.
        // This is intentionally conservative: it can only cause an extra recompile, never a stale
        // hit. Raw programs strip debug info instead (see
        // `reuses_raw_cache_for_same_program_with_different_debug_info`).
        assert_ne!(
            contract_sierra_content_hash(br#"{"sierra_program":["0x1"],"debug_info":{"a":1}}"#),
            contract_sierra_content_hash(br#"{"sierra_program":["0x1"],"debug_info":{"a":2}}"#),
        );
    }
}
