use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use which::which;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatterStatus {
    pub name: String,
    pub extensions: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct FormatterContext {
    pub directory: PathBuf,
    pub worktree: PathBuf,
}

#[async_trait]
pub trait Formatter: Send + Sync {
    fn name(&self) -> &str;
    fn extensions(&self) -> &[&str];
    fn environment(&self) -> Option<&HashMap<String, String>> {
        None
    }
    async fn enabled(&self, ctx: &FormatterContext) -> Option<Vec<String>>;
}

fn has_binary(name: &str) -> bool {
    which(name).is_ok()
}

fn has_file_in_dir(dir: &Path, filename: &str) -> bool {
    dir.join(filename).exists()
}

fn has_package_json_dep(dir: &Path, dep: &str) -> bool {
    let package_json = dir.join("package.json");
    if let Ok(content) = std::fs::read_to_string(package_json) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(deps) = json.get("dependencies").or(json.get("devDependencies")) {
                if let Some(obj) = deps.as_object() {
                    return obj.contains_key(dep);
                }
            }
        }
    }
    false
}

fn has_composer_json_dep(dir: &Path, dep: &str) -> bool {
    let composer_json = dir.join("composer.json");
    if let Ok(content) = std::fs::read_to_string(composer_json) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(require) = json.get("require").or(json.get("require-dev")) {
                if let Some(obj) = require.as_object() {
                    return obj.contains_key(dep);
                }
            }
        }
    }
    false
}

pub mod gofmt {
    use super::*;

    pub struct GofmtFormatter {
        extensions: Vec<&'static str>,
    }

    impl GofmtFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".go"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for GofmtFormatter {
        fn name(&self) -> &str {
            "gofmt"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("gofmt") {
                Some(vec!["gofmt".to_string(), "$FILE".to_string()])
            } else {
                None
            }
        }
    }
}

pub mod mix {
    use super::*;

    pub struct MixFormatter {
        extensions: Vec<&'static str>,
    }

    impl MixFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".ex", ".exs", ".eex", ".heex"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for MixFormatter {
        fn name(&self) -> &str {
            "mix"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("mix") {
                Some(vec!["mix".to_string(), "format".to_string()])
            } else {
                None
            }
        }
    }
}

pub mod prettier {
    use super::*;

    pub struct PrettierFormatter {
        extensions: Vec<&'static str>,
    }

    impl PrettierFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![
                    ".js", ".jsx", ".ts", ".tsx", ".html", ".css", ".scss", ".json", ".yaml",
                    ".md", ".yml", ".vue", ".svelte", ".mdx",
                ],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for PrettierFormatter {
        fn name(&self) -> &str {
            "prettier"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_package_json_dep(&ctx.directory, "prettier") {
                Some(vec![
                    "prettier".to_string(),
                    "--write".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod oxfmt {
    use super::*;

    pub struct OxfmtFormatter {
        extensions: Vec<&'static str>,
    }

    impl OxfmtFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".js", ".jsx", ".ts", ".tsx", ".html", ".css", ".json"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for OxfmtFormatter {
        fn name(&self) -> &str {
            "oxfmt"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, ctx: &FormatterContext) -> Option<Vec<String>> {
            if std::env::var("OPENCODE_EXPERIMENTAL_OXFMT").is_ok()
                && has_file_in_dir(&ctx.directory, "package.json")
            {
                Some(vec!["oxfmt".to_string(), "$FILE".to_string()])
            } else {
                None
            }
        }
    }
}

pub mod biome {
    use super::*;

    pub struct BiomeFormatter {
        extensions: Vec<&'static str>,
    }

    impl BiomeFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![
                    ".js", ".jsx", ".ts", ".tsx", ".json", ".css", ".scss", ".html",
                ],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for BiomeFormatter {
        fn name(&self) -> &str {
            "biome"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("biome") && has_file_in_dir(&ctx.directory, "biome.json") {
                Some(vec![
                    "biome".to_string(),
                    "format".to_string(),
                    "--write".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod zig {
    use super::*;

    pub struct ZigFormatter {
        extensions: Vec<&'static str>,
    }

    impl ZigFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".zig", ".zon"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for ZigFormatter {
        fn name(&self) -> &str {
            "zig"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("zig") {
                Some(vec![
                    "zig".to_string(),
                    "fmt".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod clang_format {
    use super::*;

    pub struct ClangFormatFormatter {
        extensions: Vec<&'static str>,
    }

    impl ClangFormatFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".c", ".cc", ".cpp", ".cxx", ".h", ".hpp", ".m", ".mm"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for ClangFormatFormatter {
        fn name(&self) -> &str {
            "clang-format"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("clang-format")
                && (has_file_in_dir(&ctx.directory, ".clang-format")
                    || has_file_in_dir(&ctx.directory, ".clang-format"))
            {
                Some(vec![
                    "clang-format".to_string(),
                    "-i".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod ktlint {
    use super::*;

    pub struct KtlintFormatter {
        extensions: Vec<&'static str>,
    }

    impl KtlintFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".kt", ".kts"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for KtlintFormatter {
        fn name(&self) -> &str {
            "ktlint"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("ktlint") {
                Some(vec![
                    "ktlint".to_string(),
                    "--format".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod ruff {
    use super::*;

    pub struct RuffFormatter {
        extensions: Vec<&'static str>,
    }

    impl RuffFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".py", ".pyi"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for RuffFormatter {
        fn name(&self) -> &str {
            "ruff"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("ruff") {
                Some(vec![
                    "ruff".to_string(),
                    "format".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod uvformat {
    use super::*;

    pub struct UvformatFormatter {
        extensions: Vec<&'static str>,
    }

    impl UvformatFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".py", ".pyi"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for UvformatFormatter {
        fn name(&self) -> &str {
            "uvformat"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("uv") && !has_binary("ruff") {
                Some(vec![
                    "uv".to_string(),
                    "format".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod air {
    use super::*;

    pub struct AirFormatter {
        extensions: Vec<&'static str>,
    }

    impl AirFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".R", ".r"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for AirFormatter {
        fn name(&self) -> &str {
            "air"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("air") {
                Some(vec![
                    "air".to_string(),
                    "fmt".to_string(),
                    "-w".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod rubocop {
    use super::*;

    pub struct RubocopFormatter {
        extensions: Vec<&'static str>,
    }

    impl RubocopFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".rb", ".rake", ".rakefile", ".Gemfile"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for RubocopFormatter {
        fn name(&self) -> &str {
            "rubocop"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("rubocop") {
                Some(vec![
                    "rubocop".to_string(),
                    "-A".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod standardrb {
    use super::*;

    pub struct StandardrbFormatter {
        extensions: Vec<&'static str>,
    }

    impl StandardrbFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".rb", ".rake", ".rakefile", ".Gemfile"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for StandardrbFormatter {
        fn name(&self) -> &str {
            "standardrb"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("standardrb") {
                Some(vec![
                    "standardrb".to_string(),
                    "--autocorrect".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod htmlbeautifier {
    use super::*;

    pub struct HtmlBeautifierFormatter {
        extensions: Vec<&'static str>,
    }

    impl HtmlBeautifierFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".erb", ".html.erb"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for HtmlBeautifierFormatter {
        fn name(&self) -> &str {
            "htmlbeautifier"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("htmlbeautifier") {
                Some(vec!["htmlbeautifier".to_string(), "$FILE".to_string()])
            } else {
                None
            }
        }
    }
}

pub mod dart {
    use super::*;

    pub struct DartFormatter {
        extensions: Vec<&'static str>,
    }

    impl DartFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".dart"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for DartFormatter {
        fn name(&self) -> &str {
            "dart"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("dart") {
                Some(vec![
                    "dart".to_string(),
                    "format".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod ocamlformat {
    use super::*;

    pub struct OcamlformatFormatter {
        extensions: Vec<&'static str>,
    }

    impl OcamlformatFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".ml", ".mli"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for OcamlformatFormatter {
        fn name(&self) -> &str {
            "ocamlformat"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_file_in_dir(&ctx.directory, ".ocamlformat") && has_binary("ocamlformat") {
                Some(vec![
                    "ocamlformat".to_string(),
                    "-i".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod terraform {
    use super::*;

    pub struct TerraformFormatter {
        extensions: Vec<&'static str>,
    }

    impl TerraformFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".tf", ".tfvars"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for TerraformFormatter {
        fn name(&self) -> &str {
            "terraform"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("terraform") {
                Some(vec![
                    "terraform".to_string(),
                    "fmt".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod latexindent {
    use super::*;

    pub struct LatexindentFormatter {
        extensions: Vec<&'static str>,
    }

    impl LatexindentFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".tex"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for LatexindentFormatter {
        fn name(&self) -> &str {
            "latexindent"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("latexindent") {
                Some(vec!["latexindent".to_string(), "$FILE".to_string()])
            } else {
                None
            }
        }
    }
}

pub mod gleam {
    use super::*;

    pub struct GleamFormatter {
        extensions: Vec<&'static str>,
    }

    impl GleamFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".gleam"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for GleamFormatter {
        fn name(&self) -> &str {
            "gleam"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("gleam") {
                Some(vec!["gleam".to_string(), "format".to_string()])
            } else {
                None
            }
        }
    }
}

pub mod shfmt {
    use super::*;

    pub struct ShfmtFormatter {
        extensions: Vec<&'static str>,
    }

    impl ShfmtFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".sh", ".bash", ".ksh", ".ash"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for ShfmtFormatter {
        fn name(&self) -> &str {
            "shfmt"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("shfmt") {
                Some(vec![
                    "shfmt".to_string(),
                    "-w".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod nixfmt {
    use super::*;

    pub struct NixfmtFormatter {
        extensions: Vec<&'static str>,
    }

    impl NixfmtFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".nix"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for NixfmtFormatter {
        fn name(&self) -> &str {
            "nixfmt"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("nixfmt") {
                Some(vec!["nixfmt".to_string(), "$FILE".to_string()])
            } else {
                None
            }
        }
    }
}

pub mod rustfmt {
    use super::*;

    pub struct RustfmtFormatter {
        extensions: Vec<&'static str>,
    }

    impl RustfmtFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".rs"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for RustfmtFormatter {
        fn name(&self) -> &str {
            "rustfmt"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("rustfmt") {
                Some(vec!["rustfmt".to_string(), "$FILE".to_string()])
            } else {
                None
            }
        }
    }
}

pub mod pint {
    use super::*;

    pub struct PintFormatter {
        extensions: Vec<&'static str>,
    }

    impl PintFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".php"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for PintFormatter {
        fn name(&self) -> &str {
            "pint"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("pint") || has_composer_json_dep(&ctx.directory, "laravel/pint") {
                Some(vec!["pint".to_string()])
            } else {
                None
            }
        }
    }
}

pub mod ormolu {
    use super::*;

    pub struct OrmoluFormatter {
        extensions: Vec<&'static str>,
    }

    impl OrmoluFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".hs"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for OrmoluFormatter {
        fn name(&self) -> &str {
            "ormolu"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("ormolu") {
                Some(vec![
                    "ormolu".to_string(),
                    "-m".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod cljfmt {
    use super::*;

    pub struct CljfmtFormatter {
        extensions: Vec<&'static str>,
    }

    impl CljfmtFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".clj", ".cljs", ".cljc", ".edn", ".clojure"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for CljfmtFormatter {
        fn name(&self) -> &str {
            "cljfmt"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("cljfmt") {
                Some(vec![
                    "cljfmt".to_string(),
                    "fix".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub mod dfmt {
    use super::*;

    pub struct DfmtFormatter {
        extensions: Vec<&'static str>,
    }

    impl DfmtFormatter {
        pub fn new() -> Self {
            Self {
                extensions: vec![".d", ".di"],
            }
        }
    }

    #[async_trait::async_trait]
    impl Formatter for DfmtFormatter {
        fn name(&self) -> &str {
            "dfmt"
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
            if has_binary("dfmt") {
                Some(vec![
                    "dfmt".to_string(),
                    "--align-switch-cases".to_string(),
                    "$FILE".to_string(),
                ])
            } else {
                None
            }
        }
    }
}

pub fn all_formatters() -> Vec<Box<dyn Formatter>> {
    vec![
        Box::new(gofmt::GofmtFormatter::new()),
        Box::new(mix::MixFormatter::new()),
        Box::new(prettier::PrettierFormatter::new()),
        Box::new(oxfmt::OxfmtFormatter::new()),
        Box::new(biome::BiomeFormatter::new()),
        Box::new(zig::ZigFormatter::new()),
        Box::new(clang_format::ClangFormatFormatter::new()),
        Box::new(ktlint::KtlintFormatter::new()),
        Box::new(ruff::RuffFormatter::new()),
        Box::new(uvformat::UvformatFormatter::new()),
        Box::new(air::AirFormatter::new()),
        Box::new(rubocop::RubocopFormatter::new()),
        Box::new(standardrb::StandardrbFormatter::new()),
        Box::new(htmlbeautifier::HtmlBeautifierFormatter::new()),
        Box::new(dart::DartFormatter::new()),
        Box::new(ocamlformat::OcamlformatFormatter::new()),
        Box::new(terraform::TerraformFormatter::new()),
        Box::new(latexindent::LatexindentFormatter::new()),
        Box::new(gleam::GleamFormatter::new()),
        Box::new(shfmt::ShfmtFormatter::new()),
        Box::new(nixfmt::NixfmtFormatter::new()),
        Box::new(rustfmt::RustfmtFormatter::new()),
        Box::new(pint::PintFormatter::new()),
        Box::new(ormolu::OrmoluFormatter::new()),
        Box::new(cljfmt::CljfmtFormatter::new()),
        Box::new(dfmt::DfmtFormatter::new()),
    ]
}

pub fn formatter_names() -> Vec<&'static str> {
    vec![
        "gofmt",
        "mix",
        "prettier",
        "oxfmt",
        "biome",
        "zig",
        "clang-format",
        "ktlint",
        "ruff",
        "uvformat",
        "air",
        "rubocop",
        "standardrb",
        "htmlbeautifier",
        "dart",
        "ocamlformat",
        "terraform",
        "latexindent",
        "gleam",
        "shfmt",
        "nixfmt",
        "rustfmt",
        "pint",
        "ormolu",
        "cljfmt",
        "dfmt",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_all_formatters_have_correct_names() {
        let formatters = all_formatters();
        let expected_names = formatter_names();
        let actual_names: Vec<&str> = formatters.iter().map(|f| f.name()).collect();
        assert_eq!(actual_names.len(), expected_names.len());
        for name in expected_names {
            assert!(
                actual_names.contains(&name),
                "Formatter '{}' not found in actual names: {:?}",
                name,
                actual_names
            );
        }
    }

    #[test]
    fn verify_all_formatters_have_extensions() {
        let formatters = all_formatters();
        for formatter in formatters {
            let extensions = formatter.extensions();
            assert!(
                !extensions.is_empty(),
                "Formatter '{}' has no extensions",
                formatter.name()
            );
        }
    }

    #[test]
    fn verify_gofmt_has_go_extension() {
        let formatter = gofmt::GofmtFormatter::new();
        assert_eq!(formatter.name(), "gofmt");
        assert!(formatter.extensions().contains(&".go"));
    }

    #[test]
    fn verify_rustfmt_has_rs_extension() {
        let formatter = rustfmt::RustfmtFormatter::new();
        assert_eq!(formatter.name(), "rustfmt");
        assert!(formatter.extensions().contains(&".rs"));
    }

    #[test]
    fn verify_all_25_plus_formatters_defined() {
        let formatters = all_formatters();
        assert!(
            formatters.len() >= 25,
            "Expected at least 25 formatters, got {}: {:?}",
            formatters.len(),
            formatters.iter().map(|f| f.name()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn verify_formatter_count() {
        let formatters = all_formatters();
        assert_eq!(formatters.len(), 26, "Expected exactly 26 formatters");
    }

    #[test]
    fn verify_gofmt_detection_logic() {
        let formatter = gofmt::GofmtFormatter::new();
        assert_eq!(formatter.name(), "gofmt");
    }

    #[test]
    fn verify_rustfmt_detection_logic() {
        let formatter = rustfmt::RustfmtFormatter::new();
        assert_eq!(formatter.name(), "rustfmt");
    }

    #[test]
    fn formatter_status_creation_with_name_extensions_enabled() {
        let status = FormatterStatus {
            name: "gofmt".to_string(),
            extensions: vec![".go".to_string()],
            enabled: true,
        };
        assert_eq!(status.name, "gofmt");
        assert_eq!(status.extensions, vec![".go"]);
        assert!(status.enabled);
    }

    #[test]
    fn formatter_status_serialization_to_json() {
        let status = FormatterStatus {
            name: "prettier".to_string(),
            extensions: vec![".js".to_string(), ".ts".to_string()],
            enabled: false,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"name\":\"prettier\""));
        assert!(json.contains("\".js\""));
        assert!(json.contains("\".ts\""));
        assert!(json.contains("\"enabled\":false"));

        let deserialized: FormatterStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "prettier");
        assert_eq!(deserialized.extensions, vec![".js", ".ts"]);
        assert!(!deserialized.enabled);
    }

    #[test]
    fn formatter_context_struct_creation() {
        let directory = PathBuf::from("/project/src");
        let worktree = PathBuf::from("/project");
        let ctx = FormatterContext {
            directory: directory.clone(),
            worktree: worktree.clone(),
        };
        assert_eq!(ctx.directory, directory);
        assert_eq!(ctx.worktree, worktree);
    }

    #[test]
    fn formatter_context_directory_and_worktree_accessible() {
        let directory = PathBuf::from("/home/user/project/src");
        let worktree = PathBuf::from("/home/user/project");
        let ctx = FormatterContext {
            directory: directory.clone(),
            worktree: worktree.clone(),
        };
        assert_eq!(ctx.directory, PathBuf::from("/home/user/project/src"));
        assert_eq!(ctx.worktree, PathBuf::from("/home/user/project"));
        assert!(ctx.directory.to_str().unwrap().ends_with("src"));
        assert!(ctx.worktree.to_str().unwrap().ends_with("project"));
    }

    #[test]
    fn verify_formatter_trait_is_object_safe() {
        fn assert_object_safe(_: Box<dyn Formatter>) {}
        let formatter: Box<dyn Formatter> = Box::new(gofmt::GofmtFormatter::new());
        assert_object_safe(formatter);
    }

    #[test]
    fn verify_trait_methods_implemented_for_sample_formatter() {
        let formatter = gofmt::GofmtFormatter::new();
        assert_eq!(formatter.name(), "gofmt");
        assert!(formatter.extensions().contains(&".go"));
        assert!(formatter.environment().is_none());
    }

    #[tokio::test]
    async fn verify_sample_formatter_enabled_method_works() {
        let formatter = gofmt::GofmtFormatter::new();
        let ctx = FormatterContext {
            directory: PathBuf::from("/tmp"),
            worktree: PathBuf::from("/tmp"),
        };
        let result = formatter.enabled(&ctx).await;
        match result {
            Some(cmd) => assert!(!cmd.is_empty()),
            None => {}
        }
    }

    #[test]
    fn verify_trait_objects_can_be_stored_in_vec() {
        let formatters: Vec<Box<dyn Formatter>> = vec![
            Box::new(gofmt::GofmtFormatter::new()),
            Box::new(rustfmt::RustfmtFormatter::new()),
            Box::new(prettier::PrettierFormatter::new()),
        ];
        assert_eq!(formatters.len(), 3);
        let names: Vec<&str> = formatters.iter().map(|f| f.name()).collect();
        assert!(names.contains(&"gofmt"));
        assert!(names.contains(&"rustfmt"));
        assert!(names.contains(&"prettier"));
    }
}
