// Copied from cairo compiler cairo-lang-compiler/src/project.rs
use cairo_lang_compiler::project::ProjectError;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use cairo_lang_defs::ids::ModuleId;
use cairo_lang_filesystem::db::FilesGroupEx;
use cairo_lang_filesystem::ids::{CrateId, CrateLongId, Directory};
pub use cairo_lang_project::*;
use cairo_lang_semantic::db::SemanticGroup;

pub const PHANTOM_PACKAGE_NAME_PREFIX: &str = "___PREFIX_FOR_PACKAGE___";

/// Setup to 'db' to compile the file at the given path.
/// Returns the id of the generated crate.
pub fn setup_single_file_project(
    db: &mut dyn SemanticGroup,
    path: &Path,
    package_name: &str,
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
        // region: Modified code
        let crate_id = db.intern_crate(CrateLongId::Real(package_name.into()));
        db.set_crate_root(crate_id, Some(Directory::Real(file_dir.to_path_buf())));
        // endregion
        Ok(crate_id)
    } else {
        // If file_stem is not lib, create a fake lib file.
        // region: Modified code
        let phantom_package_name = format!("{PHANTOM_PACKAGE_NAME_PREFIX}{file_stem}").into();
        let crate_id = db.intern_crate(CrateLongId::Real(phantom_package_name));
        // endregion
        db.set_crate_root(
            crate_id,
            Some(Directory::Real(path.parent().unwrap().to_path_buf())),
        );

        let module_id = ModuleId::CrateRoot(crate_id);
        let file_id = db.module_main_file(module_id).unwrap();

        // region: Modified code
        let file_content = fs::read_to_string(path).expect("Failed to read file at path: {path}");
        db.as_files_group_mut()
            .override_file_content(file_id, Some(Arc::new(file_content)));
        // endregion
        Ok(crate_id)
    }
}
