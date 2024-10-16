use std::path::PathBuf;

/// A build script to parse unicode range json and generate a rust file with
/// those ranges, but only if their corresponding feature is enabled.
use serde_json;

const CRATE_ROOT: &str = env!("CARGO_MANIFEST_DIR");
/// Git submodule path to the unicode range json file.
const JSON_PATH: &str = "unicode-range-json/unicode-ranges.json";
/// Rust file to generate with the unicode ranges.
const RANGES_RS: &str = "src/ranges.rs";
/// Cargo.toml content, so we can generate the features
const CARGO_TOML: &str = r#"# WARNING: This file is generated by build.rs
[package]
name = "langsan"
version = "0.0.4"
edition = "2021"
authors = ["Michael de Gans <michael.john.degans@gmail.com>"]
description = "A library for sanitizing language model input and output."
homepage = "https://github.com/mdegans/langsan"
repository = "https://github.com/mdegans/langsan"
readme = "README.md"
keywords = ["sanitization", "language", "model"]
categories = [
    "text-processing",
]
license = "MIT"

[dependencies]
serde = { version = "1", features = ["derive"], optional = true }

[build-dependencies]
serde_json = "1"
serde = { version = "1", features = ["derive"] }
static_assertions = "1"

[dev-dependencies]
serde_json = "1"

[features]
default = []
cow = []
verbose = []
serde = ["dep:serde"]

# Languages
english = []
spanish = ["latin-1-supplement"]
french = ["latin-1-supplement"]
german = ["latin-1-supplement"]
italian = ["latin-1-supplement"]
dutch = ["latin-1-supplement"]
portuguese = ["latin-1-supplement"]
russian = ["cyrillic"]
emoji = ["emoticons-emoji"]

# Unicode ranges. Note that whitespace and basic-latin are enabled by default.
# "tags" are included for completion sake but very much not recommended for use.
"#;

#[derive(serde::Deserialize)]
struct NamedRange {
    category: String,
    range: [u32; 2],
}

/// Returns `(ranges.rs, Cargo.toml)`. We have a lot of features to generate
/// so we don't want to write them all out
fn gen_ranges(json: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let ranges: Vec<NamedRange> = serde_json::from_str(json)?;
    let features: Vec<String> = ranges
        .iter()
        .map(|range| {
            range
                .category
                .to_lowercase()
                .replace(' ', "-")
                .replace('(', "")
                .replace(')', "")
        })
        .collect();
    let const_names: Vec<String> = features
        .iter()
        .map(|feature| feature.to_uppercase().replace('-', "_"))
        .collect();
    let mut cargo_toml = CARGO_TOML.to_string();
    let mut code = r#"// WARNING: This file is generated by build.rs
// Do not modify this file directly.
/// Unicode ranges
use core::ops::RangeInclusive;

// Constants for unicode ranges

/// Whitespace other than space
pub const WHITESPACE: RangeInclusive<u32> = 0x00009..=0x0000C;
/// Basic latin, excluding control characters
pub const BASIC_LATIN: RangeInclusive<u32> = 0x00020..=0x0007E; // 0x7F is DEL
"#
    .to_string();

    // The `cfg` attribute is not supported on expressions, so we have to
    // generate constants for each feature.

    for ((feature, range), const_name) in features
        .iter()
        .zip(ranges.iter())
        .zip(const_names.iter())
        .skip(2)
    {
        code.push_str(&format!("/// {}\n", range.category));
        code.push_str(&format!("#[cfg(feature = \"{feature}\")]\n",));
        code.push_str(&format!(
            "pub const {}: RangeInclusive<u32> = {:#07X}..={:#07X};\n",
            const_name, range.range[0], range.range[1]
        ));

        cargo_toml.push_str(&format!("{feature} = []\n",));
    }

    code.push_str(
        r#"/// Enabled unicode ranges.
pub const ENABLED_RANGES: &[RangeInclusive<u32>] = &[
    WHITESPACE,
    BASIC_LATIN,
"#,
    );

    for (feature, const_name) in features.iter().zip(const_names.iter()).skip(2) {
        code.push_str(&format!("    #[cfg(feature = \"{feature}\")]\n",));
        code.push_str(&format!("    {},\n", const_name));
    }

    code.push_str("];\n");

    Ok((code, cargo_toml))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Input json file
    let json_path = PathBuf::from(CRATE_ROOT).join(JSON_PATH);
    // Output `ranges.rs` file
    let ranges_path = PathBuf::from(CRATE_ROOT).join(RANGES_RS);
    // Output `Cargo.toml` file (breaks crates.io)
    // let cargo_toml_path = PathBuf::from(CRATE_ROOT).join("Cargo.toml");

    let json = std::fs::read_to_string(json_path)?;
    let (ranges_rs, _cargo_toml) = gen_ranges(&json)?;
    std::fs::write(ranges_path, ranges_rs)?;
    // std::fs::write(cargo_toml_path, cargo_toml)?;
    Ok(())
}
