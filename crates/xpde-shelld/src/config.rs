#[derive(Debug, Clone)]
pub struct ShellConfig {
    pub session_name: String,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            session_name: "xpde".to_string(),
        }
    }
}
