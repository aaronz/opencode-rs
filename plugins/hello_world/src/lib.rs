use wasmi::Engine;

pub struct HelloWorldPlugin {
    engine: Engine,
}

impl HelloWorldPlugin {
    pub fn new() -> Self {
        Self {
            engine: Engine::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = HelloWorldPlugin::new();
        assert!(std::mem::size_of_val(&plugin.engine) > 0);
    }
}
