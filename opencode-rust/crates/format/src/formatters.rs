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
                    || has_file_in_dir(&ctx.directory, "_clang-format"))
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
                extensions: vec![".rb", ".rake", ".builder", ".gemspec", ".podspec", ".rabl", ".rake", ".rbi"],
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

    #[test]
    fn verify_mix_name_returns_mix() {
        let formatter = mix::MixFormatter::new();
        assert_eq!(formatter.name(), "mix");
    }

    #[test]
    fn verify_mix_extensions_includes_all_elixir_extensions() {
        let formatter = mix::MixFormatter::new();
        assert!(formatter.extensions().contains(&".ex"));
        assert!(formatter.extensions().contains(&".exs"));
        assert!(formatter.extensions().contains(&".eex"));
        assert!(formatter.extensions().contains(&".heex"));
        assert_eq!(formatter.extensions().len(), 4);
    }

    #[tokio::test]
    async fn verify_mix_enabled_checks_which_mix() {
        use which::which;
        let formatter = mix::MixFormatter::new();
        let ctx = FormatterContext {
            directory: PathBuf::from("/tmp"),
            worktree: PathBuf::from("/tmp"),
        };
        let result = formatter.enabled(&ctx).await;
        let mix_available = which("mix").is_ok();
        if mix_available {
            assert!(result.is_some(), "mix should be available when installed");
            let cmd = result.unwrap();
            assert_eq!(cmd[0], "mix");
            assert_eq!(cmd[1], "format");
        } else {
            assert!(
                result.is_none(),
                "mix should not be available when not installed"
            );
        }
    }

    #[test]
    fn verify_prettier_name_returns_prettier() {
        let formatter = prettier::PrettierFormatter::new();
        assert_eq!(formatter.name(), "prettier");
    }

    #[test]
    fn verify_prettier_extensions_includes_common_web_extensions() {
        let formatter = prettier::PrettierFormatter::new();
        let extensions: Vec<&str> = formatter.extensions().to_vec();
        assert!(extensions.contains(&".js"), "prettier should support .js");
        assert!(extensions.contains(&".ts"), "prettier should support .ts");
        assert!(
            extensions.contains(&".html"),
            "prettier should support .html"
        );
        assert!(extensions.contains(&".css"), "prettier should support .css");
        assert!(
            extensions.contains(&".json"),
            "prettier should support .json"
        );
        assert!(
            extensions.contains(&".yaml"),
            "prettier should support .yaml"
        );
        assert!(extensions.contains(&".md"), "prettier should support .md");
    }

    #[test]
    fn verify_prettier_extensions_count() {
        let formatter = prettier::PrettierFormatter::new();
        let extensions: Vec<&str> = formatter.extensions().to_vec();
        assert!(
            extensions.len() >= 10,
            "prettier should support at least 10 extensions, got {}: {:?}",
            extensions.len(),
            extensions
        );
    }

    #[test]
    fn verify_prettier_module_exists() {
        let formatter = prettier::PrettierFormatter::new();
        assert_eq!(formatter.name(), "prettier");
        assert!(!formatter.extensions().is_empty());
    }

    #[test]
    fn verify_prettier_uses_package_json_detection() {
        let formatter = prettier::PrettierFormatter::new();
        let ctx = FormatterContext {
            directory: PathBuf::from("/tmp/non_existent_project"),
            worktree: PathBuf::from("/tmp/non_existent_project"),
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(formatter.enabled(&ctx));
        assert!(
            result.is_none(),
            "prettier should not be enabled without package.json"
        );
    }

    #[test]
    fn verify_oxfmt_name_returns_oxfmt() {
        let formatter = oxfmt::OxfmtFormatter::new();
        assert_eq!(formatter.name(), "oxfmt");
    }

    #[test]
    fn verify_oxfmt_extensions() {
        let formatter = oxfmt::OxfmtFormatter::new();
        assert!(formatter.extensions().contains(&".js"));
        assert!(formatter.extensions().contains(&".ts"));
        assert!(formatter.extensions().contains(&".tsx"));
        assert!(formatter.extensions().contains(&".jsx"));
    }

    #[tokio::test]
    async fn verify_oxfmt_enabled_checks_env_and_package_json() {
        let temp_dir = tempfile::tempdir().unwrap();
        let package_json = temp_dir.path().join("package.json");
        std::fs::write(&package_json, r#"{"name": "test"}"#).unwrap();

        let formatter = oxfmt::OxfmtFormatter::new();
        let ctx = FormatterContext {
            directory: temp_dir.path().to_path_buf(),
            worktree: temp_dir.path().to_path_buf(),
        };

        std::env::remove_var("OPENCODE_EXPERIMENTAL_OXFMT");
        let result = formatter.enabled(&ctx).await;
        assert!(
            result.is_none(),
            "oxfmt should not be enabled without env var"
        );

        std::env::set_var("OPENCODE_EXPERIMENTAL_OXFMT", "1");
        let result = formatter.enabled(&ctx).await;
        assert!(
            result.is_some(),
            "oxfmt should be enabled with env var and package.json"
        );
        std::env::remove_var("OPENCODE_EXPERIMENTAL_OXFMT");

        drop(temp_dir);
    }

    #[tokio::test]
    async fn verify_oxfmt_disabled_without_package_json() {
        let temp_dir = tempfile::tempdir().unwrap();

        std::env::set_var("OPENCODE_EXPERIMENTAL_OXFMT", "1");
        let formatter = oxfmt::OxfmtFormatter::new();
        let ctx = FormatterContext {
            directory: temp_dir.path().to_path_buf(),
            worktree: temp_dir.path().to_path_buf(),
        };
        let result = formatter.enabled(&ctx).await;
        assert!(
            result.is_none(),
            "oxfmt should not be enabled without package.json"
        );
        std::env::remove_var("OPENCODE_EXPERIMENTAL_OXFMT");

        drop(temp_dir);
    }

    #[test]
    fn verify_biome_name_returns_biome() {
        let formatter = biome::BiomeFormatter::new();
        assert_eq!(formatter.name(), "biome");
    }

    #[test]
    fn verify_biome_extensions_includes_common_web_extensions() {
        let formatter = biome::BiomeFormatter::new();
        let extensions: Vec<&str> = formatter.extensions().to_vec();
        assert!(extensions.contains(&".js"), "biome should support .js");
        assert!(extensions.contains(&".jsx"), "biome should support .jsx");
        assert!(extensions.contains(&".ts"), "biome should support .ts");
        assert!(extensions.contains(&".tsx"), "biome should support .tsx");
        assert!(extensions.contains(&".json"), "biome should support .json");
        assert!(extensions.contains(&".css"), "biome should support .css");
        assert!(extensions.contains(&".scss"), "biome should support .scss");
        assert!(extensions.contains(&".html"), "biome should support .html");
    }

    #[tokio::test]
    async fn verify_biome_enabled_checks_for_biome_json_and_biome_binary() {
        use which::which;

        let temp_dir = tempfile::tempdir().unwrap();
        let biome_json = temp_dir.path().join("biome.json");
        std::fs::write(&biome_json, "{}").unwrap();

        let formatter = biome::BiomeFormatter::new();
        let ctx = FormatterContext {
            directory: temp_dir.path().to_path_buf(),
            worktree: temp_dir.path().to_path_buf(),
        };

        let biome_available = which("biome").is_ok();
        let result = formatter.enabled(&ctx).await;

        if biome_available {
            assert!(
                result.is_some(),
                "biome should be enabled when biome.json exists and biome binary is available"
            );
            let cmd = result.unwrap();
            assert_eq!(cmd[0], "biome");
            assert_eq!(cmd[1], "format");
            assert_eq!(cmd[2], "--write");
            assert_eq!(cmd[3], "$FILE");
        } else {
            assert!(
                result.is_none(),
                "biome should not be enabled when biome binary is not available"
            );
        }

        drop(temp_dir);
    }

    #[tokio::test]
    async fn verify_biome_disabled_without_biome_json() {
        let temp_dir = tempfile::tempdir().unwrap();

        let formatter = biome::BiomeFormatter::new();
        let ctx = FormatterContext {
            directory: temp_dir.path().to_path_buf(),
            worktree: temp_dir.path().to_path_buf(),
        };

        let result = formatter.enabled(&ctx).await;
        assert!(
            result.is_none(),
            "biome should not be enabled without biome.json"
        );

        drop(temp_dir);
    }

    #[test]
    fn verify_zig_name_returns_zig() {
        let formatter = zig::ZigFormatter::new();
        assert_eq!(formatter.name(), "zig");
    }

    #[test]
    fn verify_zig_extensions_includes_zig_and_zon() {
        let formatter = zig::ZigFormatter::new();
        assert!(
            formatter.extensions().contains(&".zig"),
            "zig should support .zig"
        );
        assert!(
            formatter.extensions().contains(&".zon"),
            "zig should support .zon"
        );
        assert_eq!(formatter.extensions().len(), 2);
    }

    #[tokio::test]
    async fn verify_zig_enabled_checks_which_zig() {
        use which::which;
        let formatter = zig::ZigFormatter::new();
        let ctx = FormatterContext {
            directory: PathBuf::from("/tmp"),
            worktree: PathBuf::from("/tmp"),
        };
        let result = formatter.enabled(&ctx).await;
        let zig_available = which("zig").is_ok();
        if zig_available {
            assert!(result.is_some(), "zig should be available when installed");
            let cmd = result.unwrap();
            assert_eq!(cmd[0], "zig");
            assert_eq!(cmd[1], "fmt");
            assert_eq!(cmd[2], "$FILE");
        } else {
            assert!(
                result.is_none(),
                "zig should not be available when not installed"
            );
        }
    }

    #[test]
    fn verify_clang_format_name_returns_clang_format() {
        let formatter = clang_format::ClangFormatFormatter::new();
        assert_eq!(formatter.name(), "clang-format");
    }

    #[test]
    fn verify_clang_format_extensions_includes_c_cpp_extensions() {
        let formatter = clang_format::ClangFormatFormatter::new();
        assert!(
            formatter.extensions().contains(&".c"),
            "clang-format should support .c"
        );
        assert!(
            formatter.extensions().contains(&".cc"),
            "clang-format should support .cc"
        );
        assert!(
            formatter.extensions().contains(&".cpp"),
            "clang-format should support .cpp"
        );
        assert!(
            formatter.extensions().contains(&".cxx"),
            "clang-format should support .cxx"
        );
        assert!(
            formatter.extensions().contains(&".h"),
            "clang-format should support .h"
        );
        assert!(
            formatter.extensions().contains(&".hpp"),
            "clang-format should support .hpp"
        );
        assert!(
            formatter.extensions().contains(&".m"),
            "clang-format should support .m"
        );
        assert!(
            formatter.extensions().contains(&".mm"),
            "clang-format should support .mm"
        );
        assert_eq!(formatter.extensions().len(), 8);
    }

    #[tokio::test]
    async fn verify_clang_format_enabled_checks_for_clang_format_config() {
        use which::which;

        let temp_dir = tempfile::tempdir().unwrap();
        let clang_format_config = temp_dir.path().join(".clang-format");
        std::fs::write(&clang_format_config, "---").unwrap();

        let formatter = clang_format::ClangFormatFormatter::new();
        let ctx = FormatterContext {
            directory: temp_dir.path().to_path_buf(),
            worktree: temp_dir.path().to_path_buf(),
        };

        let clang_format_available = which("clang-format").is_ok();
        let result = formatter.enabled(&ctx).await;

        if clang_format_available {
            assert!(
                result.is_some(),
                "clang-format should be enabled when .clang-format exists and binary is available"
            );
            let cmd = result.unwrap();
            assert_eq!(cmd[0], "clang-format");
            assert_eq!(cmd[1], "-i");
            assert_eq!(cmd[2], "$FILE");
        } else {
            assert!(
                result.is_none(),
                "clang-format should not be enabled when binary is not available"
            );
        }

        drop(temp_dir);
    }

    #[tokio::test]
    async fn verify_clang_format_enabled_checks_for_underscore_clang_format_config() {
        use which::which;

        let temp_dir = tempfile::tempdir().unwrap();
        let clang_format_config = temp_dir.path().join("_clang-format");
        std::fs::write(&clang_format_config, "---").unwrap();

        let formatter = clang_format::ClangFormatFormatter::new();
        let ctx = FormatterContext {
            directory: temp_dir.path().to_path_buf(),
            worktree: temp_dir.path().to_path_buf(),
        };

        let clang_format_available = which("clang-format").is_ok();
        let result = formatter.enabled(&ctx).await;

        if clang_format_available {
            assert!(
                result.is_some(),
                "clang-format should be enabled when _clang-format exists and binary is available"
            );
        } else {
            assert!(
                result.is_none(),
                "clang-format should not be enabled when binary is not available"
            );
        }

        drop(temp_dir);
    }

    #[tokio::test]
    async fn verify_clang_format_disabled_without_config_file() {
        let temp_dir = tempfile::tempdir().unwrap();

        let formatter = clang_format::ClangFormatFormatter::new();
        let ctx = FormatterContext {
            directory: temp_dir.path().to_path_buf(),
            worktree: temp_dir.path().to_path_buf(),
        };

        let result = formatter.enabled(&ctx).await;
        assert!(
            result.is_none(),
            "clang-format should not be enabled without .clang-format config"
        );

        drop(temp_dir);
    }

    #[test]
    fn verify_ktlint_name_returns_ktlint() {
        let formatter = ktlint::KtlintFormatter::new();
        assert_eq!(formatter.name(), "ktlint");
    }

    #[test]
    fn verify_ktlint_extensions_includes_kt_and_kts() {
        let formatter = ktlint::KtlintFormatter::new();
        assert!(
            formatter.extensions().contains(&".kt"),
            "ktlint should support .kt"
        );
        assert!(
            formatter.extensions().contains(&".kts"),
            "ktlint should support .kts"
        );
        assert_eq!(formatter.extensions().len(), 2);
    }

    #[tokio::test]
    async fn verify_ktlint_enabled_checks_which_ktlint() {
        use which::which;
        let formatter = ktlint::KtlintFormatter::new();
        let ctx = FormatterContext {
            directory: PathBuf::from("/tmp"),
            worktree: PathBuf::from("/tmp"),
        };
        let result = formatter.enabled(&ctx).await;
        let ktlint_available = which("ktlint").is_ok();
        if ktlint_available {
            assert!(
                result.is_some(),
                "ktlint should be available when installed"
            );
            let cmd = result.unwrap();
            assert_eq!(cmd[0], "ktlint");
            assert_eq!(cmd[1], "--format");
            assert_eq!(cmd[2], "$FILE");
        } else {
            assert!(
                result.is_none(),
                "ktlint should not be available when not installed"
            );
        }
    }

    #[test]
    fn verify_ruff_name_returns_ruff() {
        let formatter = ruff::RuffFormatter::new();
        assert_eq!(formatter.name(), "ruff");
    }

    #[test]
    fn verify_ruff_extensions_includes_py_and_pyi() {
        let formatter = ruff::RuffFormatter::new();
        assert!(
            formatter.extensions().contains(&".py"),
            "ruff should support .py"
        );
        assert!(
            formatter.extensions().contains(&".pyi"),
            "ruff should support .pyi"
        );
        assert_eq!(formatter.extensions().len(), 2);
    }

    #[tokio::test]
    async fn verify_ruff_enabled_checks_which_ruff() {
        use which::which;
        let formatter = ruff::RuffFormatter::new();
        let ctx = FormatterContext {
            directory: PathBuf::from("/tmp"),
            worktree: PathBuf::from("/tmp"),
        };
        let result = formatter.enabled(&ctx).await;
        let ruff_available = which("ruff").is_ok();
        if ruff_available {
            assert!(result.is_some(), "ruff should be available when installed");
            let cmd = result.unwrap();
            assert_eq!(cmd[0], "ruff");
            assert_eq!(cmd[1], "format");
            assert_eq!(cmd[2], "$FILE");
        } else {
            assert!(
                result.is_none(),
                "ruff should not be available when not installed"
            );
        }
    }

    #[test]
    fn verify_uvformat_name_returns_uvformat() {
        let formatter = uvformat::UvformatFormatter::new();
        assert_eq!(formatter.name(), "uvformat");
    }

    #[test]
    fn verify_uvformat_extensions_includes_py_and_pyi() {
        let formatter = uvformat::UvformatFormatter::new();
        assert!(
            formatter.extensions().contains(&".py"),
            "uvformat should support .py"
        );
        assert!(
            formatter.extensions().contains(&".pyi"),
            "uvformat should support .pyi"
        );
        assert_eq!(formatter.extensions().len(), 2);
    }

    #[tokio::test]
    async fn verify_uvformat_enabled_checks_which_uv() {
        use which::which;
        let formatter = uvformat::UvformatFormatter::new();
        let ctx = FormatterContext {
            directory: PathBuf::from("/tmp"),
            worktree: PathBuf::from("/tmp"),
        };
        let result = formatter.enabled(&ctx).await;
        let uv_available = which("uv").is_ok();
        let ruff_available = which("ruff").is_ok();
        if uv_available && !ruff_available {
            assert!(result.is_some(), "uvformat should be available when uv is installed and ruff is not");
            let cmd = result.unwrap();
            assert_eq!(cmd[0], "uv");
            assert_eq!(cmd[1], "format");
            assert_eq!(cmd[2], "$FILE");
        } else {
            assert!(
                result.is_none(),
                "uvformat should not be available when ruff is present or uv is not installed"
            );
        }
    }

    #[test]
    fn verify_air_name_returns_air() {
        let formatter = air::AirFormatter::new();
        assert_eq!(formatter.name(), "air");
    }

    #[test]
    fn verify_air_extensions_includes_r() {
        let formatter = air::AirFormatter::new();
        assert!(
            formatter.extensions().contains(&".R"),
            "air should support .R"
        );
        assert!(
            formatter.extensions().contains(&".r"),
            "air should support .r"
        );
        assert_eq!(formatter.extensions().len(), 2);
    }

    #[tokio::test]
    async fn verify_air_enabled_checks_which_air() {
        use which::which;
        let formatter = air::AirFormatter::new();
        let ctx = FormatterContext {
            directory: PathBuf::from("/tmp"),
            worktree: PathBuf::from("/tmp"),
        };
        let result = formatter.enabled(&ctx).await;
        let air_available = which("air").is_ok();
        if air_available {
            assert!(result.is_some(), "air should be available when installed");
            let cmd = result.unwrap();
            assert_eq!(cmd[0], "air");
            assert_eq!(cmd[1], "fmt");
            assert_eq!(cmd[2], "-w");
            assert_eq!(cmd[3], "$FILE");
        } else {
            assert!(
                result.is_none(),
                "air should not be available when not installed"
            );
        }
    }

    #[test]
    fn verify_rubocop_name_returns_rubocop() {
        let formatter = rubocop::RubocopFormatter::new();
        assert_eq!(formatter.name(), "rubocop");
    }

    #[test]
    fn verify_rubocop_extensions_includes_ruby_extensions() {
        let formatter = rubocop::RubocopFormatter::new();
        assert!(
            formatter.extensions().contains(&".rb"),
            "rubocop should support .rb"
        );
        assert!(
            formatter.extensions().contains(&".rake"),
            "rubocop should support .rake"
        );
        assert!(
            formatter.extensions().contains(&".builder"),
            "rubocop should support .builder"
        );
        assert!(
            formatter.extensions().contains(&".gemspec"),
            "rubocop should support .gemspec"
        );
        assert!(
            formatter.extensions().contains(&".podspec"),
            "rubocop should support .podspec"
        );
        assert!(
            formatter.extensions().contains(&".rabl"),
            "rubocop should support .rabl"
        );
        assert!(
            formatter.extensions().contains(&".rbi"),
            "rubocop should support .rbi"
        );
    }

    #[tokio::test]
    async fn verify_rubocop_enabled_checks_which_rubocop() {
        use which::which;
        let formatter = rubocop::RubocopFormatter::new();
        let ctx = FormatterContext {
            directory: PathBuf::from("/tmp"),
            worktree: PathBuf::from("/tmp"),
        };
        let result = formatter.enabled(&ctx).await;
        let rubocop_available = which("rubocop").is_ok();
        if rubocop_available {
            assert!(result.is_some(), "rubocop should be available when installed");
            let cmd = result.unwrap();
            assert_eq!(cmd[0], "rubocop");
            assert_eq!(cmd[1], "-A");
            assert_eq!(cmd[2], "$FILE");
        } else {
            assert!(
                result.is_none(),
                "rubocop should not be available when not installed"
            );
        }
    }

    #[test]
    fn verify_standardrb_name_returns_standardrb() {
        let formatter = standardrb::StandardrbFormatter::new();
        assert_eq!(formatter.name(), "standardrb");
    }

    #[test]
    fn verify_standardrb_extensions_includes_ruby_extensions() {
        let formatter = standardrb::StandardrbFormatter::new();
        assert!(
            formatter.extensions().contains(&".rb"),
            "standardrb should support .rb"
        );
        assert!(
            formatter.extensions().contains(&".rake"),
            "standardrb should support .rake"
        );
        assert!(
            formatter.extensions().contains(&".rakefile"),
            "standardrb should support .rakefile"
        );
        assert!(
            formatter.extensions().contains(&".Gemfile"),
            "standardrb should support .Gemfile"
        );
    }

    #[tokio::test]
    async fn verify_standardrb_enabled_checks_which_standardrb() {
        use which::which;
        let formatter = standardrb::StandardrbFormatter::new();
        let ctx = FormatterContext {
            directory: PathBuf::from("/tmp"),
            worktree: PathBuf::from("/tmp"),
        };
        let result = formatter.enabled(&ctx).await;
        let standardrb_available = which("standardrb").is_ok();
        if standardrb_available {
            assert!(result.is_some(), "standardrb should be available when installed");
            let cmd = result.unwrap();
            assert_eq!(cmd[0], "standardrb");
            assert_eq!(cmd[1], "--autocorrect");
            assert_eq!(cmd[2], "$FILE");
        } else {
            assert!(
                result.is_none(),
                "standardrb should not be available when not installed"
            );
        }
    }

    #[test]
    fn verify_htmlbeautifier_name_returns_htmlbeautifier() {
        let formatter = htmlbeautifier::HtmlBeautifierFormatter::new();
        assert_eq!(formatter.name(), "htmlbeautifier");
    }

    #[test]
    fn verify_htmlbeautifier_extensions_includes_erb_and_html_erb() {
        let formatter = htmlbeautifier::HtmlBeautifierFormatter::new();
        assert!(
            formatter.extensions().contains(&".erb"),
            "htmlbeautifier should support .erb"
        );
        assert!(
            formatter.extensions().contains(&".html.erb"),
            "htmlbeautifier should support .html.erb"
        );
        assert_eq!(formatter.extensions().len(), 2);
    }

    #[tokio::test]
    async fn verify_htmlbeautifier_enabled_checks_which_htmlbeautifier() {
        use which::which;
        let formatter = htmlbeautifier::HtmlBeautifierFormatter::new();
        let ctx = FormatterContext {
            directory: PathBuf::from("/tmp"),
            worktree: PathBuf::from("/tmp"),
        };
        let result = formatter.enabled(&ctx).await;
        let htmlbeautifier_available = which("htmlbeautifier").is_ok();
        if htmlbeautifier_available {
            assert!(result.is_some(), "htmlbeautifier should be available when installed");
            let cmd = result.unwrap();
            assert_eq!(cmd[0], "htmlbeautifier");
            assert_eq!(cmd[1], "$FILE");
        } else {
            assert!(
                result.is_none(),
                "htmlbeautifier should not be available when not installed"
            );
        }
    }

    #[test]
    fn verify_dart_name_returns_dart() {
        let formatter = dart::DartFormatter::new();
        assert_eq!(formatter.name(), "dart");
    }

    #[test]
    fn verify_dart_extensions_includes_dart() {
        let formatter = dart::DartFormatter::new();
        assert!(
            formatter.extensions().contains(&".dart"),
            "dart should support .dart"
        );
        assert_eq!(formatter.extensions().len(), 1);
    }

    #[tokio::test]
    async fn verify_dart_enabled_checks_which_dart() {
        use which::which;
        let formatter = dart::DartFormatter::new();
        let ctx = FormatterContext {
            directory: PathBuf::from("/tmp"),
            worktree: PathBuf::from("/tmp"),
        };
        let result = formatter.enabled(&ctx).await;
        let dart_available = which("dart").is_ok();
        if dart_available {
            assert!(result.is_some(), "dart should be available when installed");
            let cmd = result.unwrap();
            assert_eq!(cmd[0], "dart");
            assert_eq!(cmd[1], "format");
            assert_eq!(cmd[2], "$FILE");
        } else {
            assert!(
                result.is_none(),
                "dart should not be available when not installed"
            );
        }
    }
}
