use crate::{Message, OpenCodeError, Session};

pub type Hook = Box<dyn Fn(&Session, &Message) -> Result<(), OpenCodeError> + Send + Sync>;

pub struct SessionProcessor {
    pre_hooks: Vec<Hook>,
    post_hooks: Vec<Hook>,
}

impl SessionProcessor {
    pub fn new() -> Self {
        Self {
            pre_hooks: Vec::new(),
            post_hooks: Vec::new(),
        }
    }

    pub fn add_pre_hook(&mut self, hook: Hook) {
        self.pre_hooks.push(hook);
    }

    pub fn add_post_hook(&mut self, hook: Hook) {
        self.post_hooks.push(hook);
    }

    pub fn process_message(
        &self,
        session: &Session,
        message: &Message,
    ) -> Result<(), OpenCodeError> {
        for hook in &self.pre_hooks {
            hook(session, message)?;
        }

        for hook in &self.post_hooks {
            hook(session, message)?;
        }

        Ok(())
    }
}

impl Default for SessionProcessor {
    fn default() -> Self {
        Self::new()
    }
}
