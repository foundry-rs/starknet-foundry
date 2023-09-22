// Copied from cairo compiler cairo-lang-compiler/src/project.rs
use cairo_lang_compiler::project::ProjectError;
use std::ffi::OsStr;
use std::path::Path;

use cairo_lang_filesystem::db::FilesGroupEx;
use cairo_lang_filesystem::ids::{CrateId, CrateLongId, Directory};
pub use cairo_lang_project::*;
use cairo_lang_semantic::db::SemanticGroup;

/// Setup to 'db' to compile the file at the given path.
/// Returns the id of the generated crate.
pub fn setup_single_file_project(
    db: &mut dyn SemanticGroup,
    path: &Path,
    crate_name: &str,
) -> Result<CrateId, ProjectError> {
    match path.extension().and_then(OsStr::to_str) {
        Some("cairo") => (),
        _ => {
            return Err(ProjectError::BadFileExtension);
        }
    }
    if !path.exists() {
        return Err(ProjectError::NoSuchFile {
            path: path.to_string_lossy().to_string(),
        });
    }
    let bad_path_err = || ProjectError::BadPath {
        path: path.to_string_lossy().to_string(),
    };
    let file_stem = path
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or_else(bad_path_err)?;
    if file_stem == "lib" {
        let canonical = path.canonicalize().map_err(|_| bad_path_err())?;
        let file_dir = canonical.parent().ok_or_else(bad_path_err)?;
        // region: Modified
        let crate_id = db.intern_crate(CrateLongId::Real(crate_name.into()));
        db.set_crate_root(crate_id, Some(Directory::Real(file_dir.to_path_buf())));
        // endregion
        Ok(crate_id)
    } else {
        // region: Modified code
        unreachable!();
        // endregion
    }
}
