mod node_def_builder;

use crate::package::node_def_builder::DefinitionFile;
use cdoc_base::node::definition::NodeDef;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Deserialize, Clone)]
pub struct PackageConfig {
    package: PackageInfo,
    // definitions: Vec<PathBuf>,
    #[serde(default)]
    pub dependencies: DependencyConfig,
    #[serde(default)]
    pub layouts: HashMap<String, LayoutConfig>,
}

#[derive(Deserialize, Clone)]
pub struct LayoutConfig(HashMap<String, PathBuf>);

#[derive(Deserialize, Clone, Debug, Serialize, Default)]
pub struct DependencyConfig(pub HashMap<String, DependencySpec>);

#[derive(Deserialize, Clone)]
pub struct PackageInfo {
    name: String,
    description: String,
    version: Version,
    authors: Vec<String>,
}

impl PackageConfig {
    pub fn create_main_package(dependencies: DependencyConfig) -> Self {
        PackageConfig {
            package: PackageInfo {
                name: "main".to_string(),
                description: String::default(),
                version: Version::parse("0.1.0").unwrap(),
                authors: Vec::default(),
            },

            layouts: HashMap::default(),
            dependencies,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DependencySpec {
    version: VersionReq,
    file: PathBuf,
}

impl DependencySpec {
    pub fn resolve_dependency(
        &self,
        local_path: &Path,
    ) -> anyhow::Result<(PathBuf, PackageConfig)> {
        // println!(
        //     "reading {}",
        //     local_path.as_path().join(&self.file).display()
        // );
        let config_src =
            fs::read_to_string(local_path.clone().join(&self.file).join("package.yml"))?;
        Ok((self.file.clone(), serde_yaml::from_str(&config_src)?))
    }
}

impl PackageConfig {
    pub fn resolve_dependencies(
        &self,
        local_path: &PathBuf,
    ) -> anyhow::Result<Vec<(PathBuf, PackageConfig)>> {
        self.dependencies
            .0
            .values()
            .map(|spec| spec.resolve_dependency(local_path))
            .collect()
    }

    pub fn build_module(self, local_path: &PathBuf) -> anyhow::Result<Module> {
        let dependencies = self
            .resolve_dependencies(local_path)?
            .into_iter()
            .map(|(path, pkg)| pkg.build_module(&local_path.as_path().join(path)))
            .collect::<anyhow::Result<Vec<_>>>()?;

        let definitions = dependencies
            .iter()
            .fold(DefinitionFile::default(), |acc, d| {
                acc.combine(d.node_definitions.clone())
            });

        let definitions: anyhow::Result<DefinitionFile> =
            WalkDir::new(local_path.join("templates"))
                .into_iter()
                .filter(|e| {
                    if let Ok(e) = e {
                        let path = e.path();
                        let filename = path.file_name().unwrap().to_str().unwrap();

                        // println!("path {}, ext: {:?}", path.display(), path.extension());

                        path.extension()
                            .map(|e| e.to_str().unwrap() == "yml")
                            .unwrap_or_default()
                            && filename != "project"
                            && filename != "package"
                    } else {
                        false
                    }
                })
                .try_fold(definitions, |acc, entry| {
                    let entry = entry?;
                    // let f = entry.file_name().to_str().unwrap();
                    let src = fs::read_to_string(entry.path())?;
                    let def_file: DefinitionFile = serde_yaml::from_str(&src)?;
                    // println!("deff: {:?}", &def_file);
                    Ok(acc.combine(def_file))
                });

        let node_definitions = definitions?.resolve_templates(local_path)?;
        // println!("defs: {:?}", node_definitions);

        Ok(Module {
            config: self,
            source_path: local_path.clone(),
            node_definitions,
            dependencies,
        })
    }
}

pub struct Module {
    config: PackageConfig,
    source_path: PathBuf,
    node_definitions: DefinitionFile,
    dependencies: Vec<Module>,
}

// pub struct LayoutFile(HashMap<String, >)

impl Module {
    pub fn build_defs(self) -> anyhow::Result<DefinitionFile> {
        let defs = self.node_definitions.clone();
        self.dependencies
            .into_iter()
            .try_fold(defs, |acc, m| Ok(acc.combine(m.build_defs()?)))
    }

    pub fn build_layouts(&self) -> anyhow::Result<HashMap<String, Layout>> {
        let mut layouts: HashMap<String, Layout> = self
            .dependencies
            .iter()
            .map(|m| m.build_layouts())
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();
        let new = self
            .config
            .layouts
            .iter()
            .map(|(name, layout_path)| {
                let layout = Layout(
                    layout_path
                        .0
                        .iter()
                        .map(|(prefix, path)| {
                            // println!("layout: {}, {}", prefix, path.display());
                            Ok((
                                prefix.to_string(),
                                fs::read_to_string(self.source_path.join(path))?,
                            ))
                        })
                        .collect::<anyhow::Result<HashMap<String, String>>>()?,
                );
                // println!("ll: {:?}", &layout);
                Ok((name.to_string(), layout))
            })
            .collect::<anyhow::Result<HashMap<String, Layout>>>()?;
        layouts.extend(new);
        Ok(layouts)
    }

    pub fn build_dist(self) -> anyhow::Result<Dist> {
        let layouts = self.build_layouts()?;
        // println!("l: {:?}", layouts);
        Ok(Dist {
            nodes: self.build_defs()?.try_into_node_descriptions()?,
            layouts,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout(pub(crate) HashMap<String, String>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dist {
    pub nodes: Vec<NodeDef>,
    pub layouts: HashMap<String, Layout>,
}
