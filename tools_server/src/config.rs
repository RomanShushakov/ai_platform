#[derive(Debug, Clone)]
pub struct Config {
    pub host_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host_port: std::env::var("HOST_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3001),
        }
    }
}
