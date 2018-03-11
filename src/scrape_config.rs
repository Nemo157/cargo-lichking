// Derived from
//  - https://github.com/rust-lang/cargo/blob/0.24.0/src/cargo/ops/cargo_compile.rs#L684-L818
//  - https://github.com/rust-lang/cargo/blob/0.24.0/src/cargo/ops/cargo_rustc/context.rs#L291-L375
//  - https://github.com/rust-lang/cargo/blob/0.24.0/src/cargo/ops/cargo_rustc/context.rs#L1125-L1230

use std::env;
use std::str::{self, FromStr};

use cargo::{Config, CargoResult};
use cargo::core::dependency::Platform;
use cargo::util::{Cfg, CfgExpr};
use cargo::util::errors::CargoResultExt;

pub fn scrape(config: &Config, target: Option<String>) -> CargoResult<(Platform, Vec<Cfg>)> {
    println!("target: {:?}", target);
    let host_target = config.rustc()?.host.clone();
    println!("host_target: {:?}", host_target);
    let cfg_target = config.get_string("build.target")?.map(|s| s.val);
    println!("cfg_target: {:?}", cfg_target);
    let triple = target.or(cfg_target).unwrap_or(host_target);

    println!("triple: {:?}", triple);
    let mut target_cfgs = Vec::new();

    let key = format!("target.{}", triple);
    if let Some(table) = config.get_table(&key)? {
        for (lib_name, value) in table.val {
            match lib_name.as_str() {
                "ar" | "linker" | "runner" | "rustflags" => {
                    continue
                },
                _ => {}
            }
            for (k, value) in value.table(&lib_name)?.0 {
                match &k[..] {
                    "rustc-cfg" => {
                        for &(ref cfg, _) in value.list(k)? {
                            target_cfgs.push(cfg.parse()?);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    println!("target_cfgs: {:?}", target_cfgs);
    let rustflags = {
        // First try RUSTFLAGS from the environment
        if let Ok(flags) = env::var("RUSTFLAGS") {
            flags.split(' ')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect()
        } else {
            let mut rustflags = Vec::new();

            // Then the target.*.rustflags value...
            let key = format!("target.{}.rustflags", triple);
            if let Some(args) = config.get_list_or_split_string(&key)? {
                let args = args.val.into_iter();
                rustflags.extend(args);
            }

            // ...including target.'cfg(...)'.rustflags
            if let Some(table) = config.get_table("target")? {
                let cfgs = table.val.keys().filter_map(|t| {
                    if t.starts_with("cfg(") && t.ends_with(')') {
                        let cfg = &t[4..t.len() - 1];
                        CfgExpr::from_str(cfg)
                            .ok()
                            .and_then(|c| if c.matches(&target_cfgs) { Some(t) } else { None })
                    } else {
                        None
                    }
                });
                for n in cfgs {
                    let key = format!("target.{}.rustflags", n);
                    if let Some(args) = config.get_list_or_split_string(&key)? {
                        let args = args.val.into_iter();
                        rustflags.extend(args);
                    }
                }
            }

            if rustflags.is_empty() {
                // Then the build.rustflags value
                if let Some(args) = config.get_list_or_split_string("build.rustflags")? {
                    rustflags.extend(args.val);
                }
            }

            rustflags
        }
    };

    println!("rustflags: {:?}", rustflags);
    let output = config.rustc()?
        .process()
        .arg("-")
        .arg("--crate-name").arg("___")
        .arg("--target").arg(&triple)
        .arg("--print=cfg")
        .args(&rustflags)
        .env_remove("RUST_LOG")
        .exec_with_output()
        .chain_err(|| {
            "failed to run `rustc` to learn about target-specific information"
        })?;

    // let error = str::from_utf8(&output.stderr).unwrap();
    let output = str::from_utf8(&output.stdout).unwrap();
    println!("output: {:?}", output);
    let cfg = output.lines().map(Cfg::from_str).collect::<Result<Vec<_>, _>>()?;

    println!("cfg: {:?}", cfg);
    Ok((triple.parse()?, cfg))
}
