#![allow(unknown_lints)] // for clippy

#[macro_use] extern crate clap;
extern crate cargo;
extern crate regex;
extern crate void;

mod bundle;
mod check;
mod discovery;
mod license;
mod licensed;
mod list;
mod load;
mod options;
mod scrape_config;
mod thirdparty;

use std::process;

use cargo::{Config, CliResult};

use options::{Options, Cmd};

fn main() {
    let matches = Options::app(false).get_matches();
    let options = Options::from_matches(&matches);
    let mut config = Config::default().expect("No idea why this would fail");
    let result = real_main(options, &mut config);
    if let Err(err) = result {
        config.shell().error(err).expect("Can't do much");
        process::exit(1);
    }
}

fn real_main(options: Options, config: &mut Config) -> CliResult {
    config.configure(
        options.verbose,
        Some(options.quiet),
        &options.color,
        options.frozen,
        options.locked,
        &[])?;

    config.shell().warn("IANAL: This is not legal advice and is not guaranteed to be correct.")?;

    let manifest_path = options.manifest_path;

    match options.cmd {
        Cmd::Check { package, target } => {
            let mut error = Ok(());
            let roots = load::resolve_roots(manifest_path.clone(), config, package)?;
            for root in roots {
                let packages = load::resolve_packages(manifest_path.clone(), config, vec![&root], target.clone())?;
                if let Err(err) = check::run(&root, packages, config) {
                    error = Err(err);
                }
            }
            error?;
        }

        Cmd::List { by, package, target } => {
            let roots = load::resolve_roots(manifest_path.clone(), config, package)?;
            let packages = load::resolve_packages(manifest_path, config, &roots, target)?;
            list::run(packages, by)?;
        }

        Cmd::Bundle { variant, package, target } => {
            let roots = load::resolve_roots(manifest_path.clone(), config, package)?;
            let packages = load::resolve_packages(manifest_path, config, &roots, target)?;
            bundle::run(&roots, packages, config, variant)?;
        }

        Cmd::ThirdParty { full } => {
            println!("cargo-lichking uses some third party libraries under their own license terms:");
            println!();
            for krate in thirdparty::CRATES {
                print!(" * {} under the terms of {}", krate.name, krate.licenses.name);
                if full {
                    println!(":");
                    let mut first = true;
                    for license in krate.licenses.licenses {
                        if first {
                            first = false;
                        } else {
                            println!();
                            println!("    ===============");
                        }
                        println!();
                        if let Some(text) = license.text {
                            for line in text.lines() {
                                println!("    {}", line);
                            }
                        } else {
                            println!("    Missing {} license text", license.name);
                        }
                    }
                }
                println!();
            }
        }
    }

    Ok(())
}
