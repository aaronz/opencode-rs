use std::io;

use async_trait::async_trait;
use clap::{Args, Subcommand};
use opencode_agent::{
    Agent, AgentType, BuildAgent, DebugAgent, ExploreAgent, GeneralAgent, PlanAgent, RefactorAgent,
    ReviewAgent,
};
use opencode_core::{Config, Message, OpenCodeError, Session};
use opencode_llm::provider::sealed;
use opencode_llm::provider_abstraction::{ProviderManager, ProviderSpec, DynProvider};
use opencode_llm::{ChatMessage, ChatResponse, Model, Provider, StreamingCallback};
use opencode_tools::ToolRegistry;

use crate::output::NdjsonSerializer;

struct ProviderWrapper {
    inner: DynProvider,
}

impl ProviderWrapper {
    fn new(inner: DynProvider) -> Self {
        Self { inner }
    }
}

impl sealed::Sealed for ProviderWrapper {}

#[async_trait]
impl Provider for ProviderWrapper {
    async fn complete(
        &self,
        prompt: &str,
        context: Option<&str>,
    ) -> Result<String, OpenCodeError> {
        self.inner.complete(prompt, context).await
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        self.inner.complete_streaming(prompt, callback).await
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, OpenCodeError> {
        self.inner.chat(messages).await
    }

    fn get_models(&self) -> Vec<Model> {
        self.inner.get_models()
    }

    fn provider_name(&self) -> &str {
        self.inner.provider_name()
    }
}

#[derive(Args, Debug)]
pub(crate) struct AgentArgs {
    #[command(subcommand)]
    pub action: Option<AgentAction>,
    #[arg(short, long, help = "Show detailed agent information")]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub(crate) enum AgentAction {
    List,
    Run {
        #[arg(short = 'g', long)]
        agent_name: String,
        #[arg(short, long)]
        prompt: String,
    },
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_args_no_action() {
        let args = AgentArgs { action: None, verbose: false };
        assert!(args.action.is_none());
        assert!(!args.verbose);
    }

    #[test]
    fn test_agent_args_verbose() {
        let args = AgentArgs { action: None, verbose: true };
        assert!(args.verbose);
    }

    #[test]
    fn test_agent_action_list() {
        let action = AgentAction::List;
        assert!(matches!(action, AgentAction::List));
    }

    #[test]
    fn test_agent_action_run_fields() {
        let action = AgentAction::Run {
            agent_name: "expert".to_string(),
            prompt: "Fix the bug".to_string(),
        };
        assert!(matches!(action, AgentAction::Run { .. }));
    }

    #[test]
    fn test_agent_registry_lookup_build() {
        let agent = create_agent_by_type("build").expect("build agent should exist");
        assert_eq!(agent.name(), "build");
    }

    #[test]
    fn test_agent_registry_lookup_plan() {
        let agent = create_agent_by_type("plan").expect("plan agent should exist");
        assert_eq!(agent.name(), "plan");
    }

    #[test]
    fn test_agent_registry_lookup_general() {
        let agent = create_agent_by_type("general").expect("general agent should exist");
        assert_eq!(agent.name(), "general");
    }

    #[test]
    fn test_agent_registry_lookup_explore() {
        let agent = create_agent_by_type("explore").expect("explore agent should exist");
        assert_eq!(agent.name(), "explore");
    }

    #[test]
    fn test_agent_registry_lookup_debug() {
        let agent = create_agent_by_type("debug").expect("debug agent should exist");
        assert_eq!(agent.name(), "debug");
    }

    #[test]
    fn test_agent_registry_lookup_refactor() {
        let agent = create_agent_by_type("refactor").expect("refactor agent should exist");
        assert_eq!(agent.name(), "refactor");
    }

    #[test]
    fn test_agent_registry_lookup_review() {
        let agent = create_agent_by_type("review").expect("review agent should exist");
        assert_eq!(agent.name(), "review");
    }

    #[test]
    fn test_agent_registry_lookup_invalid() {
        let result = create_agent_by_type("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_registered_agents_returns_all() {
        let agents = get_registered_agents();
        let names: Vec<&str> = agents.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"build"));
        assert!(names.contains(&"plan"));
        assert!(names.contains(&"general"));
        assert!(names.contains(&"explore"));
        assert!(names.contains(&"debug"));
        assert!(names.contains(&"refactor"));
        assert!(names.contains(&"review"));
    }

    #[test]
    fn test_get_registered_agents_count() {
        let agents = get_registered_agents();
        assert_eq!(agents.len(), 7);
    }

    #[test]
    fn test_agent_info_has_description() {
        let agents = get_registered_agents();
        for agent in agents {
            assert!(!agent.description.is_empty());
        }
    }

    #[test]
    fn test_agent_info_has_type() {
        let agents = get_registered_agents();
        for agent in agents {
            assert_eq!(agent.agent_type.to_string(), agent.name);
        }
    }

    #[test]
    fn test_format_capabilities_includes_tools() {
        let agents = get_registered_agents();
        for agent in &agents {
            let caps = format_capabilities(agent);
            assert!(caps.contains("tools"));
        }
    }
}

fn load_config() -> Config {
    let path = Config::config_path();
    Config::load(&path).unwrap_or_default()
}

fn create_agent_by_type(agent_type: &str) -> Option<Box<dyn Agent>> {
    match agent_type.to_lowercase().as_str() {
        "build" => Some(Box::new(BuildAgent::new())),
        "plan" => Some(Box::new(PlanAgent::new())),
        "general" => Some(Box::new(GeneralAgent::new())),
        "explore" => Some(Box::new(ExploreAgent::new())),
        "debug" => Some(Box::new(DebugAgent::new())),
        "refactor" => Some(Box::new(RefactorAgent::new())),
        "review" => Some(Box::new(ReviewAgent::new())),
        _ => None,
    }
}

fn get_provider_from_config(config: &Config, model: &str) -> Option<ProviderWrapper> {
    let provider_manager = ProviderManager::new();

    let (provider_type, model_name) = if model.contains('/') {
        let parts: Vec<&str> = model.split('/').collect();
        (parts[0].to_string(), parts[1].to_string())
    } else {
        ("openai".to_string(), model.to_string())
    };

    let api_key = std::env::var(format!("{}_API_KEY", provider_type.to_uppercase()))
        .ok()
        .or_else(|| {
            config
                .get_provider(&provider_type)
                .and_then(|p| p.options.as_ref())
                .and_then(|o| o.api_key.clone())
        })
        .unwrap_or_default();

    let base_url = config
        .get_provider(&provider_type)
        .and_then(|p| p.options.as_ref())
        .and_then(|o| o.base_url.clone());

    let spec = match provider_type.as_str() {
        "openai" => ProviderSpec::OpenAI {
            api_key,
            model: model_name,
            base_url,
        },
        "anthropic" => ProviderSpec::Anthropic {
            api_key,
            model: model_name,
            base_url,
        },
        "google" => ProviderSpec::Google {
            api_key,
            model: model_name,
        },
        "ollama" => ProviderSpec::Ollama {
            base_url,
            model: model_name,
        },
        "lmstudio" => ProviderSpec::LmStudio {
            base_url,
            model: model_name,
        },
        _ => ProviderSpec::OpenAI {
            api_key,
            model: model.to_string(),
            base_url: None,
        },
    };

    provider_manager.create_provider(&spec).ok().map(ProviderWrapper::new)
}

fn run_agent_with_ndjson(
    agent: Box<dyn Agent>,
    provider: ProviderWrapper,
    prompt: &str,
    model: &str,
) -> io::Result<()> {
    let mut serializer = NdjsonSerializer::stdout();
    serializer.write_start(model)?;

    let mut session = Session::new();
    session.add_message(Message::user(prompt.to_string()));

    let rt = tokio::runtime::Runtime::new().unwrap();

    let result = rt.block_on(async {
        agent.run(&mut session, &provider, &ToolRegistry::new()).await
    });

    match result {
        Ok(response) => {
            serializer.write_message("assistant", &response.content)?;
            for tool_call in response.tool_calls {
                let args_str = serde_json::to_string(&tool_call.arguments).unwrap_or_default();
                serializer.write_tool_call(&tool_call.name, &args_str)?;
            }
            serializer.write_done()?;
        }
        Err(e) => {
            serializer.write_error(&e.to_string())?;
        }
    }

    serializer.flush()?;
    Ok(())
}

pub(crate) fn run(args: AgentArgs) {
    match &args.action {
        Some(AgentAction::List) => {
            list_agents(args.verbose);
        }
        Some(AgentAction::Run {
            agent_name,
            prompt,
        }) => {
            run_agent(agent_name, prompt);
        }
        None => {
            println!("Agent command requires an action. Use 'agent list' or 'agent run'.");
        }
    }
}

fn list_agents(verbose: bool) {
    let agents = get_registered_agents();

    if verbose {
        let name = "NAME";
        let desc = "DESCRIPTION";
        let caps = "CAPABILITIES";
        println!("{:<12} {:<40} {}", name, desc, caps);
        println!("{}", "-".repeat(63));
        for agent in agents {
            let capabilities = format_capabilities(&agent);
            println!("{:<12} {:<40} {}", agent.name, agent.description, capabilities);
        }
    } else {
        println!("Available agents:");
        for agent in agents {
            println!("  {:<10} - {}", agent.name, agent.description);
        }
    }
}

fn run_agent(agent_name: &str, prompt: &str) {
    let config = load_config();
    let model = config
        .model
        .clone()
        .unwrap_or_else(|| "gpt-4o".to_string());

    let agent = match create_agent_by_type(agent_name) {
        Some(a) => a,
        None => {
            eprintln!("Error: Unknown agent type '{}'", agent_name);
            eprintln!("Available agents: build, plan, general, explore, debug, refactor, review");
            std::process::exit(1);
        }
    };

    let provider = match get_provider_from_config(&config, &model) {
        Some(p) => p,
        None => {
            eprintln!("Error: Failed to create LLM provider for model '{}'", model);
            std::process::exit(1);
        }
    };

    if let Err(e) = run_agent_with_ndjson(agent, provider, prompt, &model) {
        eprintln!("Error running agent: {}", e);
        std::process::exit(1);
    }
}

struct AgentInfo {
    name: String,
    description: String,
    #[allow(dead_code)]
    agent_type: AgentType,
}

fn get_registered_agents() -> Vec<AgentInfo> {
    vec![
        AgentInfo {
            name: "build".to_string(),
            description: "Build agent for code generation tasks".to_string(),
            agent_type: AgentType::Build,
        },
        AgentInfo {
            name: "plan".to_string(),
            description: "Plan agent for task planning".to_string(),
            agent_type: AgentType::Plan,
        },
        AgentInfo {
            name: "general".to_string(),
            description: "General agent for multi-step searches and research".to_string(),
            agent_type: AgentType::General,
        },
        AgentInfo {
            name: "explore".to_string(),
            description: "Explore agent for codebase exploration".to_string(),
            agent_type: AgentType::Explore,
        },
        AgentInfo {
            name: "debug".to_string(),
            description: "Debug agent for troubleshooting".to_string(),
            agent_type: AgentType::Debug,
        },
        AgentInfo {
            name: "refactor".to_string(),
            description: "Refactor agent for code improvements".to_string(),
            agent_type: AgentType::Refactor,
        },
        AgentInfo {
            name: "review".to_string(),
            description: "Review agent for code review".to_string(),
            agent_type: AgentType::Review,
        },
    ]
}

fn format_capabilities(info: &AgentInfo) -> String {
    let mut caps = Vec::new();
    caps.push("tools".to_string());

    if let Some(agent) = create_agent_by_type(&info.name) {
        if agent.can_write_files() {
            caps.push("write".to_string());
        }
        if agent.can_run_commands() {
            caps.push("run".to_string());
        }
    }

    caps.join(", ")
}