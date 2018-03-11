#![allow(large_enum_variant)]

use std::str::FromStr;

use cargo::core::PackageIdSpec;
use cargo::core::dependency::Platform;
use clap::{ App, Arg, SubCommand, AppSettings, ArgMatches };

#[derive(Copy, Clone, Debug)]
pub enum By {
    License,
    Crate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedPackage {
    All,
    Default,
    Specific(PackageIdSpec),
}

#[derive(Clone, Debug)]
pub enum SelectedTarget {
    All,
    Default,
    Specific(Platform),
}

#[derive(Clone, Debug)]
pub enum Bundle {
    Inline {
        file: Option<String>,
    },
    NameOnly {
        file: Option<String>,
    },
    Source {
        file: Option<String>,
    },
    Split {
        file: Option<String>,
        dir: String,
    },
}

#[derive(Clone, Debug)]
pub enum Cmd {
    List {
        by: By,
        package: SelectedPackage,
        target: SelectedTarget,
    },
    Check {
        package: SelectedPackage,
        target: SelectedTarget,
    },
    Bundle {
        variant: Bundle,
        package: SelectedPackage,
        target: SelectedTarget,
    },
    ThirdParty {
        full: bool,
    },
}

#[derive(Clone, Debug)]
pub struct Options {
    pub verbose: u32,
    pub quiet: bool,
    pub manifest_path: Option<String>,
    pub color: Option<String>,
    pub frozen: bool,
    pub locked: bool,
    pub cmd: Cmd,
}

impl By {
    fn args() -> Vec<Arg<'static, 'static>> {
        vec![
            Arg::with_name("by")
                .long("by")
                .takes_value(true)
                .possible_values(&["license", "crate"])
                .default_value("license")
                .help("Whether to list crates per license or licenses per crate"),
        ]
    }

    fn from_matches(matches: &ArgMatches) -> By {
        matches.value_of("by")
            .expect("defaulted")
            .parse()
            .expect("constrained")
    }
}

impl SelectedPackage {
    fn args() -> Vec<Arg<'static, 'static>> {
        vec![
            Arg::with_name("all")
                .long("all")
                .help("Apply to all packages in workspace"),
            Arg::with_name("package")
                .short("p").long("package")
                .takes_value(true).value_name("SPEC")
                .validator(|s| PackageIdSpec::parse(&s).map(|_| ()).map_err(|e| e.to_string()))
                .help("Package to apply this command to"),
        ]
    }

    fn help() -> &'static str {
        "\
            If the --package argument is given, then SPEC is a package id \
            specification which indicates which package this command should \
            apply to. If it is not given, then the current package is used. \
            For more information on SPEC and its format, see the `cargo help \
            pkgid` command.

\
            All packages in the workspace are used if the `--all` flag is supplied. \
            The `--all` flag may be supplied in the presence of a virtual manifest. \
        "
    }

    fn from_matches(matches: &ArgMatches) -> SelectedPackage {
        if matches.is_present("all") {
            SelectedPackage::All
        } else {
            matches.value_of("package")
                .map(|s| PackageIdSpec::parse(s).expect("validated"))
                .map(SelectedPackage::Specific)
                .unwrap_or(SelectedPackage::Default)
        }
    }
}

impl SelectedTarget {
    fn args() -> Vec<Arg<'static, 'static>> {
        vec![
            Arg::with_name("all-targets")
                .long("all-targets")
                .help("Include all optional dependencies for all targets"),
            Arg::with_name("target")
                .long("target")
                .takes_value(true).value_name("TRIPLE")
                .help("Include optional dependencies for the specified triple"),
        ]
    }

    fn from_matches(matches: &ArgMatches) -> SelectedTarget {
        if matches.is_present("all-targets") {
            SelectedTarget::All
        } else {
            matches.value_of("target")
                .map(str::to_owned)
                .map(Platform::Name)
                .map(SelectedTarget::Specific)
                .unwrap_or(SelectedTarget::Default)
        }
    }
}

impl Bundle {
    fn args() -> Vec<Arg<'static, 'static>> {
        vec![
            Arg::with_name("variant")
                .long("variant")
                .takes_value(true)
                .possible_values(&["inline", "name-only", "source", "split"])
                .default_value("inline")
                .requires_if("split", "dir")
                .help("")
                .long_help("\
What sort of bundle to produce:

    inline:
        Output a single file to location specified by --file containing the
        name and content of the license used by each dependency

    name-only:
        Output a single file to location specified by --file containing just
        the name of the license used by each dependency

    source:
        Output a single file to location specified by --file containing Rust
        source with the name and content of the license used by each dependency

    split:
        Output a file to location specified by --file containing the name of
        the license used by each dependency, along with a folder at the location
        specified by --dir containing the text of each dependency's license in a
        separate file inside

\
                "),
            Arg::with_name("file")
                .long("file")
                .takes_value(true).value_name("FILE")
                .help("The file to output to (standard out if not specified)"),
            Arg::with_name("dir")
                .long("dir")
                .takes_value(true).value_name("DIR")
                .help("The directory to output to"),
        ]
    }

    fn from_matches(matches: &ArgMatches) -> Bundle {
        match matches.value_of("variant").expect("defaulted") {
            "inline" => Bundle::Inline {
                file: matches.value_of("file").map(ToOwned::to_owned),
            },
            "name-only" => Bundle::NameOnly {
                file: matches.value_of("file").map(ToOwned::to_owned),
            },
            "source" => Bundle::Source {
                file: matches.value_of("file").map(ToOwned::to_owned),
            },
            "split" => Bundle::Split {
                file: matches.value_of("file").map(ToOwned::to_owned),
                dir: matches.value_of("dir").expect("required").to_owned(),
            },
            variant => panic!("Unexpected variant value {}", variant),
        }
    }
}

impl Options {
    pub fn app(subcommand_required: bool) -> App<'static, 'static> {
        App::new("cargo")
            .bin_name("cargo")
            .subcommand(Options::subapp(subcommand_required))
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .global_settings(&[
                AppSettings::ColorAuto,
                AppSettings::ColoredHelp,
                AppSettings::VersionlessSubcommands,
                AppSettings::DeriveDisplayOrder,
                AppSettings::UnifiedHelpMessage,
            ])
    }

    // For some reason setting SubcommandRequired on the "lichking" sub command
    // propogates down to its subcommands as well, need to work out what's
    // happening and open a clap ticket so this argument is not needed.
    //
    // For now, try parsing the args without the subcommand being required,
    // then if we don't get a subcommand re-parse with it required to get the
    // error output.
    pub fn subapp(subcommand_required: bool) -> App<'static, 'static> {
        let mut app = SubCommand::with_name("lichking")
            .author(crate_authors!())
            .version(crate_version!())
            .about(crate_description!())
            .args(&Options::args())
            .subcommands(Options::subcommands());
        if subcommand_required {
            app = app.setting(AppSettings::SubcommandRequiredElseHelp);
        }
        app
    }

    pub fn args() -> Vec<Arg<'static, 'static>> {
        vec![
            Arg::with_name("verbose")
                .short("v").long("verbose")
                .multiple(true)
                .help("Use verbose output (-vv very verbose output)"),
            Arg::with_name("quiet")
                .short("q").long("quiet")
                .help("Use quiet output"),
            Arg::with_name("manifest-path")
                .long("manifest-path")
                .takes_value(true).value_name("PATH")
                .help("Path to the manifest to analyze"),
            Arg::with_name("color")
                .long("color")
                .takes_value(true).value_name("COLOR")
                .possible_values(&["auto", "always", "never"])
                .help("Coloring"),
            Arg::with_name("frozen")
                .long("frozen")
                .help("Require Cargo.lock and cache are up to date"),
            Arg::with_name("locked")
                .long("locked")
                .help("Require Cargo.lock is up to date"),
        ]
    }

    pub fn subcommands() -> Vec<App<'static, 'static>> {
        vec![
            SubCommand::with_name("check")
                .about("Check that all dependencies have a compatible license with a package")
                .args(&SelectedPackage::args())
                .args(&SelectedTarget::args())
                .after_help(SelectedPackage::help())


            SubCommand::with_name("list")
                .about("List licensing of all dependencies")
                .args(&By::args())
                .args(&SelectedPackage::args())
                .args(&SelectedTarget::args())
                .after_help(SelectedPackage::help())

            SubCommand::with_name("bundle")
                .about("Bundle all dependencies licenses ready for distribution")
                .args(&Bundle::args())
                .args(&SelectedPackage::args())
                .args(&SelectedTarget::args())
                .after_help(SelectedPackage::help())

            SubCommand::with_name("thirdparty")
                .about("List dependencies of cargo-lichking")
                .args(&[
                    Arg::with_name("full")
                        .long("full")
                        .help("Whether to list license content for each dependency"),
                ]),
        ]
    }

    pub fn from_matches(matches: &ArgMatches) -> Options {
        let matches = matches.subcommand_matches("lichking").expect("required");
        Options {
            verbose: matches.occurrences_of("verbose") as u32,
            quiet: matches.is_present("quiet"),
            manifest_path: matches.value_of("manifest-path").map(ToOwned::to_owned),
            color: matches.value_of("color").map(ToOwned::to_owned),
            frozen: matches.is_present("frozen"),
            locked: matches.is_present("locked"),
            cmd: match matches.subcommand() {
                ("check", Some(matches)) => {
                    Cmd::Check {
                        package: SelectedPackage::from_matches(matches),
                        target: SelectedTarget::from_matches(matches),
                    }
                }
                ("list", Some(matches)) => {
                    Cmd::List {
                        by: By::from_matches(matches),
                        package: SelectedPackage::from_matches(matches),
                        target: SelectedTarget::from_matches(matches),
                    }
                }
                ("bundle", Some(matches)) => {
                    Cmd::Bundle {
                        variant: Bundle::from_matches(matches),
                        package: SelectedPackage::from_matches(matches),
                        target: SelectedTarget::from_matches(matches),
                    }
                }
                ("thirdparty", Some(matches)) => {
                    Cmd::ThirdParty {
                        full: matches.is_present("full"),
                    }
                }
                (subcommand, _) => {
                    Options::app(true).get_matches();
                    panic!("Unexpected subcommand {}", subcommand)
                }
            },
        }
    }
}

impl FromStr for By {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "license" => Ok(By::License),
            "crate" => Ok(By::Crate),
            s => Err(format!("Cannot parse By from '{}'", s)),
        }
    }
}
