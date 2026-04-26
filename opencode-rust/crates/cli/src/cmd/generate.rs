use clap::Args;
use std::path::{Path, PathBuf};

#[derive(Args, Debug)]
pub(crate) struct GenerateArgs {
    #[arg(short, long)]
    pub template: Option<String>,

    #[arg(short, long)]
    pub output: Option<String>,
}

struct TemplateInfo {
    name: &'static str,
    description: &'static str,
}

const TEMPLATES: &[TemplateInfo] = &[
    TemplateInfo {
        name: "rust-crate",
        description: "Generate a new Rust crate with Cargo.toml",
    },
    TemplateInfo {
        name: "react-component",
        description: "Generate a React TypeScript component",
    },
    TemplateInfo {
        name: "python-script",
        description: "Generate a Python script template",
    },
];

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_args_default() {
        let args = GenerateArgs {
            template: None,
            output: None,
        };
        assert!(args.template.is_none());
        assert!(args.output.is_none());
    }

    #[test]
    fn test_generate_args_with_template() {
        let args = GenerateArgs {
            template: Some("rust-crate".to_string()),
            output: None,
        };
        assert_eq!(args.template.as_deref(), Some("rust-crate"));
    }

    #[test]
    fn test_generate_args_with_output() {
        let args = GenerateArgs {
            template: None,
            output: Some("my-project".to_string()),
        };
        assert_eq!(args.output.as_deref(), Some("my-project"));
    }

    #[test]
    fn test_generate_args_full() {
        let args = GenerateArgs {
            template: Some("react-component".to_string()),
            output: Some("components/Button.tsx".to_string()),
        };
        assert_eq!(args.template.as_deref(), Some("react-component"));
        assert_eq!(args.output.as_deref(), Some("components/Button.tsx"));
    }

    #[test]
    fn test_template_list() {
        assert!(!TEMPLATES.is_empty());
        assert!(TEMPLATES.iter().any(|t| t.name == "rust-crate"));
    }
}

pub(crate) fn run(args: GenerateArgs) {
    let template_name = args.template.as_deref().unwrap_or("list");

    if template_name == "list" {
        println!("Available templates:");
        for template in TEMPLATES {
            println!("  {} - {}", template.name, template.description);
        }
        println!("\nUse --template <name> --output <path> to generate a template");
        return;
    }

    let template = match TEMPLATES.iter().find(|t| t.name == template_name) {
        Some(t) => t,
        None => {
            eprintln!(
                "Error: Unknown template '{}'. Use 'generate' without arguments to see available templates.",
                template_name
            );
            std::process::exit(1);
        }
    };

    let output_dir = args
        .output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(template.name));

    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        eprintln!("Error creating directory '{}': {}", output_dir.display(), e);
        std::process::exit(1);
    }

    let result = match template.name {
        "rust-crate" => generate_rust_crate(&output_dir),
        "react-component" => generate_react_component(&output_dir),
        "python-script" => generate_python_script(&output_dir),
        _ => {
            eprintln!("Template '{}' is not yet implemented", template.name);
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error generating template: {}", e);
        std::process::exit(1);
    }

    println!(
        "Generated template '{}' in '{}'",
        template.name,
        output_dir.display()
    );
}

fn generate_rust_crate(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(output_dir.join("Cargo.toml"), "[package]\nname = \"my-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n")?;
    println!("  Created '{}/Cargo.toml'", output_dir.display());

    std::fs::create_dir_all(output_dir.join("src"))?;
    std::fs::write(
        output_dir.join("src/lib.rs"),
        "pub fn new() {\n    // Your code here\n}\n",
    )?;
    println!("  Created '{}/src/lib.rs'", output_dir.display());

    std::fs::write(
        output_dir.join("src/main.rs"),
        "fn main() {\n    println!(\"Hello, world!\");\n}\n",
    )?;
    println!("  Created '{}/src/main.rs'", output_dir.display());

    Ok(())
}

fn generate_react_component(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(
        output_dir.join("Component.tsx"),
        "import React from 'react';\n\ninterface Props {\n  name: string;\n}\n\nexport const Component: React.FC<Props> = ({ name }) => {\n  return <div className=\"component\">{name}</div>;\n};\n",
    )?;
    println!("  Created '{}/Component.tsx'", output_dir.display());

    std::fs::write(
        output_dir.join("Component.css"),
        ".component {\n  /* Styles here */\n}\n",
    )?;
    println!("  Created '{}/Component.css'", output_dir.display());

    Ok(())
}

fn generate_python_script(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(
        output_dir.join("script.py"),
        "#!/usr/bin/env python3\n\ndef main():\n    print(\"Hello, world!\")\n\nif __name__ == \"__main__\":\n    main()\n",
    )?;
    println!("  Created '{}/script.py'", output_dir.display());

    Ok(())
}
