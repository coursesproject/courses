mod definitions;
mod node_description;

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Package {
    package: PackageInfo,
    definitions: Vec<PathBuf>,
    dependencies: HashMap<String, DependencySpec>,
}

#[derive(Deserialize)]
pub struct PackageInfo {
    name: String,
    description: String,
    version: Version,
    authors: Vec<String>,
}

#[derive(Deserialize)]
pub struct DependencySpec {
    version: VersionReq,
}

impl Package {}
