use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use cairo_lang_compiler::project::{AllCratesConfig, ProjectConfig, ProjectConfigContent};
use cairo_lang_filesystem::cfg::{Cfg as CompilerCfg, CfgSet};
use cairo_lang_filesystem::db::{
    CrateIdentifier, CrateSettings, DependencySettings, Edition, ExperimentalFeaturesConfig,
};
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use scarb_metadata::{Cfg as ScarbCfg, CompilationUnitMetadata, PackageId, PackageMetadata};
use semver::Version;
use smol_str::ToSmolStr;

/// Different targets for cairo.
pub mod targets {
    /// [lib]
    pub const LIB: &str = "lib";
    /// #[cfg(test)]
    pub const TEST: &str = "test";
    /// Starknet smart contracts
    pub const STARKNET_CONTRACT: &str = "starknet-contract";
    /// All the targets
    pub const TARGETS: [&str; 3] = [LIB, TEST, STARKNET_CONTRACT];
}

/// Converts [`&[ScarbCfg]`] to a [`CfgSet`]
pub fn to_cairo_cfg(cfgs: &[ScarbCfg]) -> CfgSet {
    let mut cfg_set = CfgSet::new();
    cfgs.iter().for_each(|cfg| match cfg {
        ScarbCfg::KV(key, value) => {
            cfg_set.insert(CompilerCfg {
                key: key.to_smolstr(),
                value: Some(value.to_smolstr()),
            });
        }
        ScarbCfg::Name(name) => {
            cfg_set.insert(CompilerCfg {
                key: name.to_smolstr(),
                value: None,
            });
        }
    });
    cfg_set
}

/// Convert a string to a compiler [`Edition`]. If the edition is unknown it'll return an error.
pub fn to_cairo_edition(edition: &str) -> Result<Edition> {
    match edition {
        "2023_01" => Ok(Edition::V2023_01),
        "2023_10" => Ok(Edition::V2023_10),
        "2023_11" => Ok(Edition::V2023_11),
        "2024_07" => Ok(Edition::V2024_07),
        _ => Err(anyhow!("Unknown edition {}", edition)),
    }
}

/// Gets a bunch of informations related to the project from several objects.
///
/// Mostly a copy pasta of
/// https://github.com/software-mansion/scarb/blob/fb34a0ce85e0a46e15f58abd3fbaaf1d3c4bf012/scarb/src/compiler/helpers.rs#L17-L62
/// but with metadata objects
pub fn build_project_config(
    compilation_unit: &CompilationUnitMetadata,
    corelib_id: &PackageId,
    package_path: PathBuf,
    edition: Edition,
    version: &Version,
    packages: &[PackageMetadata],
) -> Result<ProjectConfig> {
    let crate_roots = compilation_unit
        .components
        .iter()
        .filter(|component| &component.package != corelib_id)
        .map(|component| {
            (
                CrateIdentifier::from(&component.name),
                component.source_root().into(),
            )
        })
        .collect();
    let crates_config: OrderedHashMap<CrateIdentifier, CrateSettings> = compilation_unit
        .components
        .iter()
        .map(|component| {
            let package = packages
                .iter()
                .find(|package| package.id == component.package);
            let cfg_set = component.cfg.as_ref().map(|cfgs| to_cairo_cfg(cfgs));
            let (package_ed, dependencies) = if let Some(pack) = package {
                let mut dependencies: BTreeMap<String, DependencySettings> = pack
                    .dependencies
                    .iter()
                    .filter_map(|dependency| {
                        compilation_unit
                            .components
                            .iter()
                            .find(|compilation_unit_metadata_component| {
                                compilation_unit_metadata_component.name == dependency.name
                            })
                            .map(|compilation_unit_metadata_component| {
                                (
                                    dependency.name.clone(),
                                    DependencySettings {
                                        discriminator: compilation_unit_metadata_component
                                            .discriminator
                                            .as_ref()
                                            .map(ToSmolStr::to_smolstr),
                                    },
                                )
                            })
                    })
                    .collect();
                // Adds itself to dependencies
                dependencies.insert(
                    pack.name.clone(),
                    DependencySettings {
                        discriminator: component.discriminator.as_ref().map(ToSmolStr::to_smolstr),
                    },
                );
                (
                    pack.edition
                        .as_ref()
                        .map_or_else(|| edition, |ed| to_cairo_edition(ed).unwrap()),
                    dependencies,
                )
            } else {
                (edition, BTreeMap::default())
            };
            (
                CrateIdentifier::from(&component.name),
                CrateSettings {
                    name: Some(component.name.to_smolstr()),
                    edition: package_ed,
                    cfg_set,
                    dependencies,
                    experimental_features: ExperimentalFeaturesConfig {
                        negative_impls: package
                            .map(|package| {
                                package
                                    .experimental_features
                                    .contains(&String::from("negative_impls"))
                            })
                            .unwrap_or(false),
                        coupons: package
                            .map(|package| {
                                package
                                    .experimental_features
                                    .contains(&String::from("coupons"))
                            })
                            .unwrap_or(false),
                        associated_item_constraints: package
                            .map(|package| {
                                package
                                    .experimental_features
                                    .contains(&String::from("associated_item_constraints"))
                            })
                            .unwrap_or(false),
                    },
                    version: Some(version.clone()),
                },
            )
        })
        .collect();
    let crates_config = AllCratesConfig {
        override_map: crates_config,
        ..Default::default()
    };
    let content = ProjectConfigContent {
        crate_roots,
        crates_config,
    };

    let project_config = ProjectConfig {
        base_path: package_path,
        content,
    };
    Ok(project_config)
}
