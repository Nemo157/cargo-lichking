use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use cargo_metadata::Package;
use regex::Regex;

use crate::license::License;

const HIGH_CONFIDENCE_LIMIT: f32 = 0.10;
const LOW_CONFIDENCE_LIMIT: f32 = 0.15;

#[derive(Debug, Eq, PartialEq)]
pub enum Confidence {
    Confident,
    SemiConfident,
    Unsure,
}

#[derive(Debug, Eq, PartialEq)]
pub struct LicenseText {
    pub path: PathBuf,
    pub text: String,
    pub confidence: Confidence,
}

fn add_frequencies(freq: &mut HashMap<String, u32>, text: &str) {
    for word in Regex::new(r"\w+").unwrap().find_iter(text) {
        *freq
            .entry(word.as_str().to_lowercase().to_owned())
            .or_insert(0) += 1;
    }
}

fn calculate_frequency(text: &str) -> HashMap<String, u32> {
    let mut freq = HashMap::new();
    add_frequencies(&mut freq, text);
    freq
}

fn compare(mut text_freq: HashMap<String, u32>, template_freq: &HashMap<String, u32>) -> u32 {
    let mut errors = 0;

    for (word, &count) in template_freq {
        let text_count = text_freq.remove(word).unwrap_or(0);
        let diff = ((text_count as i32) - (count as i32)).abs() as u32;
        errors += diff;
    }

    for (_, count) in text_freq {
        errors += count;
    }

    errors
}

fn check_against_template(text: &str, license: &License) -> Confidence {
    let text_freq = calculate_frequency(text);

    let template_freq = if let License::Multiple(ref licenses) = *license {
        let mut template_freq = HashMap::new();
        for license in licenses {
            if let Some(template) = license.template() {
                add_frequencies(&mut template_freq, template)
            } else {
                return Confidence::Unsure;
            }
        }
        template_freq
    } else if let Some(template) = license.template() {
        calculate_frequency(template)
    } else {
        return Confidence::Unsure;
    };

    let total: u32 = template_freq.values().sum();
    let errors = compare(text_freq, &template_freq);
    let score = (errors as f32) / (total as f32);

    if score < HIGH_CONFIDENCE_LIMIT {
        Confidence::Confident
    } else if score < LOW_CONFIDENCE_LIMIT {
        Confidence::SemiConfident
    } else {
        Confidence::Unsure
    }
}

pub fn find_generic_license_text(
    package: &Package,
    license: &License,
) -> anyhow::Result<Option<LicenseText>> {
    fn generic_license_name(name: &str) -> bool {
        name.to_uppercase() == "LICENSE"
            || name.to_uppercase() == "LICENCE"
            || name.to_uppercase() == "LICENSE.MD"
            || name.to_uppercase() == "LICENSE.TXT"
    }

    for entry in fs::read_dir(package.manifest_path.parent().unwrap())? {
        let entry = entry?;
        let path = entry.path().to_owned();
        let name = entry.file_name().to_string_lossy().into_owned();

        if generic_license_name(&name) {
            if let Ok(text) = fs::read_to_string(&path) {
                let confidence = check_against_template(&text, license);
                return Ok(Some(LicenseText {
                    path,
                    text,
                    confidence,
                }));
            }
        }
    }

    Ok(None)
}

pub fn find_license_text(package: &Package, license: &License) -> anyhow::Result<Vec<LicenseText>> {
    fn name_matches(name: &str, license: &License) -> bool {
        let name = name.to_uppercase();
        match *license {
            License::Apache_2_0 => name == "LICENSE-APACHE",
            License::BSL_1_0 => name == "LICENSE-BOOST" || name == "LICENSE-BSL",
            License::Custom(ref custom) => {
                let custom = custom.to_uppercase();
                name == custom || name == format!("LICENSE-{}", custom)
            }
            ref license => {
                let license = license.to_string().to_uppercase();
                name == license || name == format!("LICENSE-{}", license)
            }
        }
    }

    let mut texts = Vec::new();
    for entry in fs::read_dir(package.manifest_path.parent().unwrap())? {
        let entry = entry?;
        let path = entry.path().to_owned();
        let name = entry.file_name().to_string_lossy().into_owned();

        if name_matches(&name, license) {
            if let Ok(text) = fs::read_to_string(&path) {
                let confidence = check_against_template(&text, license);
                texts.push(LicenseText {
                    path,
                    text,
                    confidence,
                });
            }
        }
    }

    Ok(texts)
}
