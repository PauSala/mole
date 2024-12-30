pub mod data;

use data::{CLockFile, CTomlFile, Dependency, FoundDependency};
use hashbrown::HashMap;
use std::fs;

use crate::{error::MoleError, file_explorer::CargoFiles};

#[derive(Default)]
pub struct FileParser;

impl FileParser {
    pub fn new() -> Self {
        FileParser
    }

    pub fn parse(
        &self,
        files: HashMap<String, CargoFiles>,
        target_dep: &str,
    ) -> Result<Vec<FoundDependency>, MoleError> {
        let mut found = Vec::new();

        for (_, package) in files {
            if let Some(ref toml) = package.ctoml {
                // Parse .toml
                let toml_file = fs::read_to_string(toml)?;
                let parsed = self.parse_toml(
                    &toml_file,
                    target_dep,
                    toml.to_str().expect("Path should be a file"),
                );

                let package_name;
                if let Some(parsed) = &parsed.iter().next() {
                    package_name = parsed.package_name.clone();
                } else {
                    package_name = self.parse_name(&toml_file);
                }

                found.extend(parsed);

                // parse .lock
                if let Some(ref lock) = package.clock {
                    let lock_file = fs::read_to_string(lock)?;
                    let parsed = self.parse_lock(
                        &lock_file,
                        target_dep,
                        lock.to_str().unwrap(),
                        package_name,
                    );
                    found.extend(parsed);
                }
            }
        }
        found.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(found)
    }

    fn parse_toml(&self, contents: &str, target_dep: &str, path: &str) -> Vec<FoundDependency> {
        let mut res = Vec::new();
        let parsed: Result<CTomlFile, _> = toml::from_str(contents);
        if let Ok(toml) = parsed {
            let package_name = toml
                .package
                .map(|package| package.name)
                .unwrap_or_else(|| "-".to_string());

            for deps in [toml.dependencies, toml.dev_dependencies] {
                if let Some(found) = self.parse_dependencies(deps, target_dep, path, &package_name)
                {
                    res.push(found);
                }
            }
            if let Some(data) = toml.target.and_then(|target| target.targets) {
                for (_, target) in data {
                    for deps in [target.dependencies, target.dev_dependencies] {
                        if let Some(found) =
                            self.parse_dependencies(deps, target_dep, path, &package_name)
                        {
                            res.push(found);
                        }
                    }
                }
            }
            return res;
        } else {
            match parsed {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Unparseable file: {:?} {e}", path);
                }
            };
        }
        res
    }

    fn parse_lock(
        &self,
        contents: &str,
        dependency_name: &str,
        path: &str,
        package_name: String,
    ) -> Vec<FoundDependency> {
        let mut res = Vec::new();
        let parsed: Result<CLockFile, _> = toml::from_str(contents);
        if let Ok(lock_file) = parsed {
            for package in lock_file.package {
                if package.name == dependency_name {
                    res.push(FoundDependency {
                        package_name: package_name.clone(),
                        dep_version: package.version,
                        path: path.to_owned(),
                    });
                }
            }
        }
        res
    }

    fn parse_name(&self, contents: &str) -> String {
        toml::from_str::<CTomlFile>(contents)
            .ok()
            .and_then(|toml| toml.package.map(|package| package.name))
            .unwrap_or_else(|| "-".to_string())
    }

    fn parse_dependencies(
        &self,
        dependencies: Option<HashMap<String, Dependency>>,
        target_dep: &str,
        path: &str,
        package_name: &str,
    ) -> Option<FoundDependency> {
        if let Some(dependencies) = dependencies {
            for (dep_name, dep) in dependencies {
                if dep_name == target_dep {
                    match dep {
                        data::Dependency::Simple(version) => {
                            return Some(FoundDependency {
                                package_name: package_name.to_owned(),
                                dep_version: version,
                                path: path.to_string(),
                            })
                        }
                        data::Dependency::Detailed(dependency_details) => {
                            return Some(FoundDependency {
                                package_name: package_name.to_owned(),
                                dep_version: dependency_details.version.unwrap_or("-".to_string()),
                                path: path.to_string(),
                            })
                        }
                    }
                }
            }
        }
        None
    }
}
