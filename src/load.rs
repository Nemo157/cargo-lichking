use std::collections::HashSet;

use cargo::core::dependency::Kind;
use cargo::core::{Package, PackageId, Workspace};
use cargo::ops;
use cargo::util::important_paths::find_root_manifest_for_wd;
use cargo::{Config, CargoResult};

use options::{SelectedPackage, SelectedTarget};
use scrape_config;

pub fn resolve_roots(
        manifest_path: Option<String>,
        config: &Config,
        package: SelectedPackage) -> CargoResult<Vec<Package>> {
    let root_manifest = find_root_manifest_for_wd(manifest_path, config.cwd())?;
    let workspace = Workspace::new(&root_manifest, config)?;

    Ok(match package {
        SelectedPackage::All => {
            workspace.members().cloned().collect()
        }
        SelectedPackage::Default => {
            vec![workspace.current()?.clone()]
        }
        SelectedPackage::Specific(spec) => {
            let (packages, _) = ops::resolve_ws(&workspace)?;
            let package_id = spec.query(packages.package_ids())?;
            vec![packages.get(package_id)?.clone()]
        }
    })
}

fn lookup_failed(dep_id: &PackageId, id: &PackageId) -> String {
    format!("Looking up a packages dependency in the package failed, failed to find '{}' in '{}'", dep_id, id)
}

pub fn resolve_packages<'a, I: IntoIterator<Item=&'a Package>>(
        manifest_path: Option<String>,
        config: &Config,
        roots: I,
        target: SelectedTarget) -> CargoResult<Vec<Package>> {
    let root_manifest = find_root_manifest_for_wd(manifest_path, config.cwd())?;
    let workspace = Workspace::new(&root_manifest, config)?;

    let (packages, resolve) = ops::resolve_ws(&workspace)?;

    let platform = match target {
        SelectedTarget::All => None,
        SelectedTarget::Default => Some(scrape_config::scrape(config, None)?),
        SelectedTarget::Specific(target) => Some(scrape_config::scrape(config, Some(target.to_string()))?),
    };

    let mut result = HashSet::new();
    let mut to_check = roots.into_iter().map(|p| p.package_id()).collect::<Vec<_>>();
    while let Some(id) = to_check.pop() {
        let package = packages.get(id)?;
        if !result.insert(package) {
            continue;
        }
        for dep_id in resolve.deps_not_replaced(id) {
            let dep = package.dependencies().iter()
                .find(|d| d.matches_id(dep_id))
                .ok_or_else(|| lookup_failed(dep_id, id))?;
            if dep.kind() != Kind::Normal  {
                continue;
            }
            if let Some((ref platform, ref cfgs)) = platform {
                if !dep.platform().map(|p| p.matches(&platform.to_string(), Some(cfgs))).unwrap_or(true) {
                    continue;
                }
            }
            let dep_id = resolve.replacement(dep_id).unwrap_or(dep_id);
            to_check.push(dep_id);
        }
    }

    Ok(result.into_iter().cloned().collect())
}
