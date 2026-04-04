pub mod graph_renderer;

pub fn format_duration(seconds: u64) -> String {
    format!("{}s", seconds)
}

pub fn get_env_var(name: &str) -> Option<String> {
    std::env::var(name).ok()
}
