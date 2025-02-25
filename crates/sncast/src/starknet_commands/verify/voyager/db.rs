use std::path::PathBuf;

use cairo_lang_compiler::project::{
    update_crate_roots_from_project_config, AllCratesConfig, ProjectConfig, ProjectConfigContent,
};
use cairo_lang_defs::db::{ext_as_virtual_impl, DefsDatabase, DefsGroup};
use cairo_lang_filesystem::db::{CrateSettings, Edition, ExperimentalFeaturesConfig};
use cairo_lang_filesystem::ids::{Directory, VirtualFile};
use cairo_lang_filesystem::{
    cfg::{Cfg, CfgSet},
    db::{init_files_group, AsFilesGroupMut, ExternalFiles, FilesDatabase, FilesGroup},
};
use cairo_lang_parser::db::{ParserDatabase, ParserGroup};
use cairo_lang_semantic::{
    db::{SemanticDatabase, SemanticGroup},
    inline_macros::get_default_plugin_suite,
    plugin::PluginSuite,
};
use cairo_lang_starknet::starknet_plugin_suite;
use cairo_lang_syntax::node::db::{SyntaxDatabase, SyntaxGroup};
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use cairo_lang_utils::Upcast;

use itertools::Itertools;
use salsa;
use scarb_metadata::{
    CompilationUnitComponentMetadata, CompilationUnitMetadata, Metadata, PackageId, PackageMetadata,
};
use smol_str::{SmolStr, ToSmolStr};

#[salsa::database(
    FilesDatabase,
    ParserDatabase,
    SyntaxDatabase,
    DefsDatabase,
    SemanticDatabase
)]
pub struct VoyagerDatabase {
    storage: salsa::Storage<Self>,
}

impl VoyagerDatabase {
    pub fn new(project_config: ProjectConfig) -> Self {
        let plugin_suite = [get_default_plugin_suite(), starknet_plugin_suite()]
            .into_iter()
            .fold(PluginSuite::default(), |mut acc, suite| {
                acc.add(suite);
                acc
            });
        let mut db = Self {
            storage: Default::default(),
        };

        init_files_group(&mut db);

        db.set_cfg_set(Self::initial_cfg_set().into());
        db.apply_plugin_suite(plugin_suite);
        db.apply_project_config(project_config);

        db
    }

    fn initial_cfg_set() -> CfgSet {
        CfgSet::from_iter([Cfg::name("voyager")])
    }

    fn apply_plugin_suite(&mut self, plugin_suite: PluginSuite) {
        self.set_macro_plugins(plugin_suite.plugins);
        self.set_inline_macro_plugins(plugin_suite.inline_macro_plugins.into());
        self.set_analyzer_plugins(plugin_suite.analyzer_plugins);
    }

    fn apply_project_config(&mut self, config: ProjectConfig) {
        update_crate_roots_from_project_config(self, &config);
    }
}

impl salsa::Database for VoyagerDatabase {}

impl ExternalFiles for VoyagerDatabase {
    fn ext_as_virtual(&self, external_id: salsa::InternId) -> VirtualFile {
        ext_as_virtual_impl(self.upcast(), external_id)
    }
}

impl salsa::ParallelDatabase for VoyagerDatabase {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(VoyagerDatabase {
            storage: self.storage.snapshot(),
        })
    }
}

impl AsFilesGroupMut for VoyagerDatabase {
    fn as_files_group_mut(&mut self) -> &mut (dyn FilesGroup + 'static) {
        self
    }
}

impl Upcast<dyn FilesGroup> for VoyagerDatabase {
    fn upcast(&self) -> &(dyn FilesGroup + 'static) {
        self
    }
}

impl Upcast<dyn ParserGroup> for VoyagerDatabase {
    fn upcast(&self) -> &(dyn ParserGroup + 'static) {
        self
    }
}

impl Upcast<dyn SyntaxGroup> for VoyagerDatabase {
    fn upcast(&self) -> &(dyn SyntaxGroup + 'static) {
        self
    }
}

impl Upcast<dyn DefsGroup> for VoyagerDatabase {
    fn upcast(&self) -> &(dyn DefsGroup + 'static) {
        self
    }
}

impl Upcast<dyn SemanticGroup> for VoyagerDatabase {
    fn upcast(&self) -> &(dyn SemanticGroup + 'static) {
        self
    }
}

const LIB_TARGET_KIND: &str = "lib";
const STARKNET_TARGET_KIND: &str = "starknet-contract";
const CORELIB_CRATE_NAME: &str = "core";

pub fn get_project_config(
    metadata: &Metadata,
    package_metadata: &PackageMetadata,
) -> ProjectConfig {
    let compilation_unit_metadata = package_compilation_unit(metadata, package_metadata.id.clone());
    let corelib = get_corelib(compilation_unit_metadata);
    let dependencies = get_dependencies(compilation_unit_metadata);
    let crates_config = get_crates_config(metadata, compilation_unit_metadata);
    ProjectConfig {
        base_path: package_metadata.root.clone().into(),
        corelib: Some(Directory::Real(corelib.source_root().into())),
        content: ProjectConfigContent {
            crate_roots: dependencies,
            crates_config,
        },
    }
}

fn package_compilation_unit(
    metadata: &Metadata,
    package_id: PackageId,
) -> &CompilationUnitMetadata {
    let relevant_cus = metadata
        .compilation_units
        .iter()
        .filter(|m| m.package == package_id)
        .collect_vec();

    relevant_cus
        .iter()
        .find(|m| m.target.kind == LIB_TARGET_KIND)
        .or_else(|| {
            relevant_cus
                .iter()
                .find(|m| m.target.kind == STARKNET_TARGET_KIND)
        })
        .expect("failed to find compilation unit for package")
}

fn get_corelib(
    compilation_unit_metadata: &CompilationUnitMetadata,
) -> &CompilationUnitComponentMetadata {
    compilation_unit_metadata
        .components
        .iter()
        .find(|du| du.name == CORELIB_CRATE_NAME)
        .expect("Corelib could not be found")
}

fn get_dependencies(
    compilation_unit_metadata: &CompilationUnitMetadata,
) -> OrderedHashMap<SmolStr, PathBuf> {
    compilation_unit_metadata
        .components
        .iter()
        .filter(|du| du.name != CORELIB_CRATE_NAME)
        .map(|cu| {
            (
                cu.name.to_smolstr(),
                cu.source_root().to_owned().into_std_path_buf(),
            )
        })
        .collect()
}

fn get_crates_config(
    metadata: &Metadata,
    compilation_unit_metadata: &CompilationUnitMetadata,
) -> AllCratesConfig {
    let crates_config: OrderedHashMap<SmolStr, CrateSettings> = compilation_unit_metadata
        .components
        .iter()
        .map(|component| {
            let pkg = metadata.get_package(&component.package).unwrap_or_else(|| {
                panic!(
                    "failed to find = {} package",
                    &component.package.to_string()
                )
            });
            (
                SmolStr::from(&component.name),
                get_crate_settings_for_package(
                    pkg,
                    component.cfg.as_ref().map(|cfg_vec| build_cfg_set(cfg_vec)),
                ),
            )
        })
        .collect();

    AllCratesConfig {
        override_map: crates_config,
        ..Default::default()
    }
}

fn get_crate_settings_for_package(
    package: &PackageMetadata,
    cfg_set: Option<CfgSet>,
) -> CrateSettings {
    let edition = package
        .edition
        .clone()
        .map_or(Edition::default(), |edition| {
            let edition_value = serde_json::Value::String(edition);
            serde_json::from_value(edition_value).unwrap()
        });

    let experimental_features = ExperimentalFeaturesConfig {
        negative_impls: package
            .experimental_features
            .contains(&String::from("negative_impls")),
        coupons: package
            .experimental_features
            .contains(&String::from("coupons")),
    };

    CrateSettings {
        edition,
        cfg_set,
        experimental_features,
        version: Some(package.version.clone()),
    }
}

fn build_cfg_set(cfg: &[scarb_metadata::Cfg]) -> CfgSet {
    CfgSet::from_iter(cfg.iter().map(|cfg| {
        serde_json::to_value(cfg)
            .and_then(serde_json::from_value::<Cfg>)
            .expect("Cairo's `Cfg` must serialize identically as Scarb Metadata's `Cfg`.")
    }))
}
