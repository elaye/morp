use std::collections::HashMap;
use std::fs::{self, DirEntry, File};
use std::io;
use std::path::{Path, PathBuf};

use petgraph::{algo, graphmap::DiGraphMap};
use serde_json;

pub struct Monorepo {
    packages: Vec<PackageJson>,
}

#[derive(Debug)]
pub enum MonorepoError {
    PackagesDirRead(io::Error),
    PackagesDirEntryRead(io::Error),
    PackageJsonRead(io::Error),
    PackageJsonDeserialization {
        error: serde_json::Error,
        path: PathBuf,
    },
    CyclicDependencies,
}

#[derive(Debug, Deserialize)]
pub struct PackageJson {
    name: String,
    dependencies: Option<HashMap<String, String>>,
}

impl Monorepo {
    pub fn load(path: &Path) -> Result<Monorepo, MonorepoError> {
        let packages_path = path.join("packages/");

        let packages = Monorepo::load_packages(&packages_path)?;

        Ok(Monorepo { packages })
    }

    fn load_packages(path: &Path) -> Result<Vec<PackageJson>, MonorepoError> {
        let entries: Vec<DirEntry> = fs::read_dir(path)
            .map_err(MonorepoError::PackagesDirRead)?
            .map(|entry| entry.map_err(MonorepoError::PackagesDirEntryRead))
            .collect::<Result<Vec<DirEntry>, MonorepoError>>()?;

        entries
            .iter()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .map(|p| Monorepo::load_package(&p))
            .collect::<Result<Vec<PackageJson>, MonorepoError>>()
    }

    fn load_package(path: &PathBuf) -> Result<PackageJson, MonorepoError> {
        let filepath = path.join("package.json");

        let json_file = File::open(filepath).map_err(MonorepoError::PackageJsonRead)?;

        serde_json::from_reader(json_file).map_err(|error| {
            MonorepoError::PackageJsonDeserialization {
                error,
                path: path.to_path_buf(),
            }
        })
    }

    pub fn get_deps_graph(&self) -> Result<DiGraphMap<&str, f32>, MonorepoError> {
        let mut graph = DiGraphMap::new();

        self.packages.iter().for_each(|package| {
            graph.add_node(package.name.as_str());
        });

        self.packages
            .iter()
            .for_each(|package| match &package.dependencies {
                None => (),
                Some(deps) => {
                    deps.keys().for_each(|dep_name| {
                        if let Some(d) = self.packages.iter().find(|d| &d.name == dep_name) {
                            graph.add_edge(&package.name, &d.name, 0.);
                        }
                    });
                }
            });

        if algo::is_cyclic_directed(&graph) {
            Err(MonorepoError::CyclicDependencies)
        } else {
            Ok(graph)
        }
    }
}
