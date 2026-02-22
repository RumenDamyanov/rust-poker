//! Server configuration parsed from environment variables.

/// Application configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Server listen port.
    pub port: u16,
    /// Server bind host.
    pub host: String,
}

impl AppConfig {
    /// Load configuration from environment variables with sensible defaults.
    pub fn from_env() -> Self {
        Self {
            port: std::env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8084),
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
        }
    }

    /// The full bind address (e.g. `0.0.0.0:8084`).
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig {
            port: 8084,
            host: "0.0.0.0".to_string(),
        };
        assert_eq!(config.port, 8084);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.bind_addr(), "0.0.0.0:8084");
    }
}
