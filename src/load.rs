use std::collections::HashSet;

use anyhow::anyhow;
use cargo_metadata::{DependencyKind, Metadata, Package};
use serde::Deserialize;

use crate::options::SelectedPackage;
use crate::query::{PackagesExt, ResolveExt};

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Workspace {
    default_members: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Manifest {
    workspace: Workspace,
}

pub fn resolve_roots(
    metadata: &Metadata,
    package: SelectedPackage,
) -> anyhow::Result<Vec<&Package>> {
    match package {
        SelectedPackage::All => metadata
            .workspace_members
            .iter()
            .map(|id| metadata.packages.by_id(id))
            .collect(),
        SelectedPackage::Default => {
            // If `metadata.resolve.root` is set that means we're in a concrete crate directory,
            // otherwise we should be at a virtual manifest and should check the default members
            let resolve = metadata
                .resolve
                .as_ref()
                .ok_or_else(|| anyhow!("Couldn't load resolve graph"))?;
            if let Some(root) = &resolve.root {
                Ok(vec![metadata.packages.by_id(root)?])
            } else {
                let manifest: Manifest = toml::from_slice(&std::fs::read({
                    let mut path = metadata.workspace_root.clone();
                    path.push("Cargo.toml");
                    path
                })?)?;
                if let Some(default_members) = manifest.workspace.default_members {
                    default_members
                        .iter()
                        .map(|name| {
                            metadata
                                .workspace_members
                                .iter()
                                .filter_map(|id| metadata.packages.by_id(id).ok())
                                .find(|p| &p.name == name)
                                .ok_or_else(|| anyhow!("Couldn't find workspace member {}", name))
                        })
                        .collect()
                } else {
                    metadata
                        .workspace_members
                        .iter()
                        .map(|id| metadata.packages.by_id(id))
                        .collect()
                }
            }
        }
        SelectedPackage::Specific(name) => Ok(vec![metadata
            .packages
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| anyhow!("Could not find package {}", name))?]),
    }
}

pub fn resolve_packages<'a>(
    metadata: &'a Metadata,
    roots: &'a [&'a Package],
) -> anyhow::Result<Vec<&'a Package>> {
    let mut result = Vec::new();
    let mut added = HashSet::new();

    let mut to_check = roots.iter().map(|p| &p.id).collect::<Vec<_>>();

    let packages = &metadata.packages;
    let resolve = metadata
        .resolve
        .as_ref()
        .ok_or_else(|| anyhow!("Couldn't load resolve graph"))?;

    while let Some(id) = to_check.pop() {
        if added.insert(id) {
            let package = packages.by_id(id)?;
            result.push(package);
            for dep in resolve.by_id(id)? {
                if dep
                    .dep_kinds
                    .iter()
                    .any(|info| info.kind == DependencyKind::Normal)
                {
                    to_check.push(&dep.pkg);
                }
            }
        }
    }

    Ok(result)
}
