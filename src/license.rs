use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use void::Void;

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
#[allow(non_camel_case_types)]
pub enum License {
    // Licenses specified in the [SPDX License List](https://spdx.org/licenses/)
    Unlicense,
    MIT,
    X11,
    BSD_2_Clause,
    BSD_3_Clause,
    Apache_2_0,
    LGPL_2_0,
    LGPL_2_1,
    LGPL_2_1Plus,
    LGPL_3_0,
    LGPL_3_0Plus,
    MPL_1_1,
    MPL_2_0,
    GPL_2_0,
    GPL_2_0Plus,
    GPL_3_0,
    GPL_3_0Plus,
    AGPL_3_0,
    AGPL_3_0Plus,

    // Special cases
    Custom(String),
    File(PathBuf),
    Multiple(Vec<License>),
    Unspecified,
}

impl Default for License {
    fn default() -> License {
        License::Unspecified
    }
}

macro_rules! compatibility {
  ($s:expr, $o:expr, { $($a:pat => [$($b:pat),+])+ }) => {
    #[allow(single_match)]
    match $s {
      $(
        $a => match $o {
          $($b)|+ => return Some(true),
          _ => (),
        }
      ),*
    }
  };
}

impl License {
    pub fn can_include(&self, other: &License) -> Option<bool> {
        use self::License::*;

        if let Unspecified = *other { return Some(false); }

        if let Custom(_) = *self { return None; }
        if let Custom(_) = *other { return None; }
        if let File(_) = *self { return None; }
        if let File(_) = *other { return None; }

        if let Multiple(ref licenses) = *self {
            for license in licenses {
                if let Some(can_include) = license.can_include(other) {
                    if !can_include {
                        return Some(false);
                    }
                } else {
                    return None;
                }
            }
            return Some(true);
        }

        if let Multiple(ref licenses) = *other {
            let mut seen_none = false;
            for license in licenses {
                if let Some(can_include) = self.can_include(license) {
                    if can_include {
                        return Some(true);
                    }
                } else {
                    seen_none = true;
                }
            }
            return if seen_none { None } else { Some(false) };
        }

        if let LGPL_2_0 = *self { return None; /* TODO: unknown */ }
        if let LGPL_2_0 = *other { return None; /* TODO: unknown */ }

        // GPL compatibility from https://www.gnu.org/licenses/gpl-faq.html#AllCompatibility
        // The fields where combination would be upgraded to GPLv3
        // are not considered.
        // The license on the left is "I want to license my code under".
        compatibility!(*self, *other, {
            Unspecified         => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause]

            LGPL_2_0     => [LGPL_2_0] // TODO: probably allows more

            Unlicense    => [Unlicense, MIT, X11]
            MIT          => [Unlicense, MIT, X11]
            X11          => [Unlicense, MIT, X11]
            BSD_2_Clause => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause]
            BSD_3_Clause => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause]
            Apache_2_0   => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause, Apache_2_0]
            MPL_1_1      => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause, MPL_1_1]
            MPL_2_0      => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause, Apache_2_0, MPL_2_0]
            LGPL_2_1Plus => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause, MPL_2_0, LGPL_2_1Plus, LGPL_2_1, LGPL_3_0]
            LGPL_2_1     => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause, MPL_2_0, LGPL_2_1Plus, LGPL_2_1, LGPL_3_0]
            LGPL_3_0Plus => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause, MPL_2_0, Apache_2_0, LGPL_2_1Plus, LGPL_3_0Plus]
            LGPL_3_0     => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause, MPL_2_0, Apache_2_0, LGPL_2_1Plus, LGPL_2_1, LGPL_3_0Plus, LGPL_3_0]
            GPL_2_0Plus  => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause, MPL_2_0, LGPL_2_1Plus, LGPL_2_1, GPL_2_0Plus]
            GPL_2_0      => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause, MPL_2_0, LGPL_2_1Plus, LGPL_2_1, GPL_2_0Plus, GPL_2_0]
            GPL_3_0Plus  => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause, MPL_2_0, Apache_2_0, LGPL_2_1Plus, LGPL_2_1, LGPL_3_0Plus, LGPL_3_0, GPL_2_0Plus, GPL_3_0Plus, GPL_3_0, AGPL_3_0]
            GPL_3_0      => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause, MPL_2_0, Apache_2_0, LGPL_2_1Plus, LGPL_2_1, GPL_2_0Plus, GPL_3_0Plus, GPL_3_0]
            AGPL_3_0Plus => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause, MPL_2_0, Apache_2_0, LGPL_2_1Plus, LGPL_2_1, LGPL_3_0Plus, LGPL_3_0, GPL_2_0Plus, GPL_3_0Plus, GPL_3_0, AGPL_3_0Plus]
            AGPL_3_0     => [Unlicense, MIT, X11, BSD_2_Clause, BSD_3_Clause, MPL_2_0, Apache_2_0, LGPL_2_1Plus, LGPL_2_1, GPL_2_0Plus, GPL_3_0Plus, GPL_3_0, AGPL_3_0Plus, AGPL_3_0]

            // TODO: These are `unreachable!()`, can't figure out a nice way to allow this in the macro...
            Custom(_)    => [MIT]
            File(_)      => [MIT]
            Multiple(_)  => [MIT]
        });

        Some(false)
    }

    pub fn template(&self) -> Option<&'static str> {
        Some(match *self {
            License::Unlicense     => include_str!("licenses/Unlicense"),
            License::MIT           => include_str!("licenses/MIT"),
            License::Apache_2_0    => include_str!("licenses/Apache-2.0"),
            License::BSD_3_Clause    => include_str!("licenses/BSD-3-Clause"),
            License::Multiple(_)   => { panic!("TODO: Refactor multiple handling") }
            _                      => { return None }
        })
    }
}

impl FromStr for License {
    type Err = Void;
    fn from_str(s: &str) -> Result<License, Void> {
        Ok(match s.trim() {
            "Unlicense"                       => License::Unlicense,
            "MIT"                             => License::MIT,
            "X11"                             => License::X11,
            "BSD-2-Clause"                    => License::BSD_2_Clause,
            "BSD-3-Clause"                    => License::BSD_3_Clause,
            "Apache-2.0"                      => License::Apache_2_0,
            "LGPL-2.0-only"     | "LGPL-2.0"  => License::LGPL_2_0,
            "LGPL-2.1-only"     | "LGPL-2.1"  => License::LGPL_2_1,
            "LGPL-2.1-or-later" | "LGPL-2.1+" => License::LGPL_2_1Plus,
            "LGPL-3.0-only"     | "LGPL-3.0"  => License::LGPL_3_0,
            "LGPL-3.0-or-later" | "LGPL-3.0+" => License::LGPL_3_0Plus,
            "MPL-1.1"                         => License::MPL_1_1,
            "MPL-2.0"                         => License::MPL_2_0,
            "GPL-2.0-only"      | "GPL-2.0"   => License::GPL_2_0,
            "GPL-2.0-or-later"  | "GPL-2.0+"  => License::GPL_2_0Plus,
            "GPL-3.0-only"      | "GPL-3.0"   => License::GPL_3_0,
            "GPL-3.0-or-later"  | "GPL-3.0+"  => License::GPL_3_0Plus,
            "AGPL-3.0-only"     | "AGPL-3.0"  => License::AGPL_3_0,
            "AGPL-3.0-or-later" | "AGPL-3.0+" => License::AGPL_3_0Plus,
            s if s.contains('/') || s.contains(" OR ") => {
                let mut licenses = s.split('/')
                    .flat_map(|s| s.split(" OR "))
                    .map(str::parse)
                    .map(Result::unwrap)
                    .collect::<Vec<License>>();
                licenses.sort();
                License::Multiple(licenses)
            },
            s => License::Custom(s.to_owned()),
        })
    }
}

impl fmt::Display for License {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            License::Unlicense     => write!(w, "Unlicense"),
            License::MIT           => write!(w, "MIT"),
            License::X11           => write!(w, "X11"),
            License::BSD_2_Clause  => write!(w, "BSD-2-Clause"),
            License::BSD_3_Clause  => write!(w, "BSD-3-Clause"),
            License::Apache_2_0    => write!(w, "Apache-2.0"),
            License::LGPL_2_0      => write!(w, "LGPL-2.0-only"),
            License::LGPL_2_1      => write!(w, "LGPL-2.1-only"),
            License::LGPL_2_1Plus  => write!(w, "LGPL-2.1-or-later"),
            License::LGPL_3_0      => write!(w, "LGPL-3.0-only"),
            License::LGPL_3_0Plus  => write!(w, "LGPL-3.0-or-later"),
            License::MPL_1_1       => write!(w, "MPL-1.1"),
            License::MPL_2_0       => write!(w, "MPL-2.0"),
            License::GPL_2_0       => write!(w, "GPL-2.0-only"),
            License::GPL_2_0Plus   => write!(w, "GPL-2.0-or-later"),
            License::GPL_3_0       => write!(w, "GPL-3.0-only"),
            License::GPL_3_0Plus   => write!(w, "GPL-3.0-or-later"),
            License::AGPL_3_0      => write!(w, "AGPL-3.0-only"),
            License::AGPL_3_0Plus  => write!(w, "AGPL-3.0-or-later"),
            License::Custom(ref s) => write!(w, "{}", s),
            License::File(ref f)   => write!(w, "License specified in file ({})", f.to_string_lossy()),
            License::Multiple(ref ls)   => {
                write!(w, "{}", ls[0])?;
                for l in ls.iter().skip(1) {
                    write!(w, " / {}", l)?;
                }
                Ok(())
            },
            License::Unspecified          => write!(w, "No license specified"),
        }
    }
}
