//! The firestream-api-server-config crate contains functionality for parsing as well as accessing the project's documentation.

use anyhow::{anyhow, Context};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::{Display, Formatter};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tracing::info;

/// The application configuration.
///
/// This struct is the central point for the entire application configuration. It holds the [`ServerConfig`] as well as [`DatabaseConfig`]and can be extended with any application-specific configuration settings that will be read from the main `app.toml` and the environment-specific configuration files.
///
/// For any setting that appears in both the `app.toml` and the environment-specific file, the latter will override the former so that default settings can be kept in `app.toml` that are overridden per environment if necessary.
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    /// the server configuration: [`ServerConfig`]
    pub server: ServerConfig,
    /// the database configuration: [`DatabaseConfig`]
    pub database: DatabaseConfig,
    // add your config settings here…
}

/// The server configuration.
///
/// This struct keeps all settings specific to the server – currently that is the interface the server binds to
/// but more might be added in the future. The struct is provided pre-defined by Gerust and cannot be changed. It
/// **must** be used for the `server` field in the application-specific [`Config`] struct:
///
/// ```rust
/// #[derive(Deserialize, Clone, Debug)]
/// pub struct Config {
///     #[serde(default)]
///     pub server: ServerConfig,
///     pub database: DatabaseConfig,
///     // add your config settings here…
/// }
/// ```
#[derive(Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ServerConfig {
    /// The port to bind to, e.g. 3000
    pub port: u16,

    /// The ip to bind to, e.g. 127.0.0.1 or ::1
    pub ip: IpAddr,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 3000,
        }
    }
}

impl ServerConfig {
    /// Returns the full address the server binds to, including both the ip and port.
    ///
    /// This can be used when creating a TCP Listener:
    ///
    /// ```rust
    /// let config: Config = load_config(Environment::Development);
    /// let listener = TcpListener::bind(&config.server.addr).await?;
    /// serve(listener, app.into_make_service()).await?;
    ///  ```
    pub fn addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }
}

/// The database configuration.
///
/// This struct keeps all settings specific to the database – currently that is the database URL to use to connect to the database
/// but more might be added in the future. The struct is provided pre-defined by Gerust and cannot be changed. It
/// **must** be used for the `database` field in the application-specific [`Config`] struct:
///
/// ```rust
/// #[derive(Deserialize, Clone, Debug)]
/// pub struct Config {
///     #[serde(default)]
///     pub server: ServerConfig,
///     pub database: DatabaseConfig,
///     // add your config settings here…
/// }
/// ```
#[derive(Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct DatabaseConfig {
    /// The URL to use to connect to the database, e.g. "postgresql://user:password@localhost:5432/database"
    pub url: String,
}

impl DatabaseConfig {
    /// Create a database URL from environment variables with fallback support
    pub fn from_env() -> Result<Self, anyhow::Error> {
        // First, check if DATABASE_URL is set (takes precedence)
        if let Ok(database_url) = env::var("DATABASE_URL") {
            return Ok(DatabaseConfig { url: database_url });
        }
        
        // Check if PG* environment variables are set (prefer these over APP_DATABASE__URL)
        if env::var("PGHOST").is_ok() || env::var("PGUSER").is_ok() {
            let host = env::var("PGHOST").unwrap_or_else(|_| "localhost".to_string());
            let port = env::var("PGPORT").unwrap_or_else(|_| "5432".to_string());
            let user = env::var("PGUSER").unwrap_or_else(|_| "postgres".to_string());
            let password = env::var("PGPASSWORD").unwrap_or_else(|_| "".to_string());
            let database = env::var("PGDATABASE").unwrap_or_else(|_| "postgres".to_string());
            
            let url = if password.is_empty() {
                format!("postgres://{}@{}:{}/{}", user, host, port, database)
            } else {
                format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, database)
            };
            
            return Ok(DatabaseConfig { url });
        }
        
        // Finally, check if APP_DATABASE__URL is set
        if let Ok(database_url) = env::var("APP_DATABASE__URL") {
            return Ok(DatabaseConfig { url: database_url });
        }
        
        // If nothing is set, return an error
        Err(anyhow!("No database configuration found. Please set DATABASE_URL, PG* environment variables, or APP_DATABASE__URL"))
    }
}

/// Loads the application configuration for a particular environment.
///
/// Configuration settings are loaded from these sources (in that order so that latter sources override former):
/// * the `config/app.toml` file
/// * the `config/environments/<development|production|test>.toml` files depending on the environment
/// * environment variables prefixed with `APP_`
///
/// Note: This function does NOT load .env files. Environment variables should be set
/// by the deployment system (Docker, systemd, shell, etc.) before the application runs.
pub fn load_config<'a, T>(env: &Environment) -> Result<T, anyhow::Error>
where
    T: Deserialize<'a>,
{

    let env_config_file = match env {
        Environment::Development => "development.toml",
        Environment::Production => "production.toml",
        Environment::Test => "test.toml",
    };

    let config: T = Figment::new()
        .merge(Serialized::defaults(ServerConfig::default()).key("server"))
        .merge(Toml::file("config/app.toml"))
        .merge(Toml::file(format!(
            "config/environments/{}",
            env_config_file
        )))
        .merge(Env::prefixed("APP_").split("__"))
        .extract()
        .context("Could not read configuration!")?;

    Ok(config)
}

/// The environment the application runs in.
///
/// The application can run in 3 different environments: development, production, and test. Depending on the environment, the configuration might be different (e.g. different databases) or the application might behave differently.
#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    /// The development environment is what developers would use locally.
    Development,
    /// The production environment would typically be used in the released, user-facing deployment of the app.
    Production,
    /// The test environment is using when running e.g. `cargo test`
    Test,
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Production => write!(f, "production"),
            Environment::Test => write!(f, "test"),
        }
    }
}

/// Returns the currently active environment.
///
/// If the `APP_ENVIRONMENT` env var is set, the application environment is parsed from that (which might fail if an invalid environment is set). If the env var is not set, [`Environment::Development`] is returned.
pub fn get_env() -> Result<Environment, anyhow::Error> {
    match env::var("APP_ENVIRONMENT") {
        Ok(val) => {
            info!(r#"Setting environment from APP_ENVIRONMENT: "{}""#, val);
            parse_env(&val)
        }
        Err(_) => {
            info!("Defaulting to environment: development");
            Ok(Environment::Development)
        }
    }
}

/// Parses an [`Environment`] from a string.
///
/// The environment can be passed in different forms, e.g. "dev", "development", "prod", etc. If an invalid environment is passed, an error is returned.
pub fn parse_env(env: &str) -> Result<Environment, anyhow::Error> {
    let env = &env.to_lowercase();
    match env.as_str() {
        "dev" => Ok(Environment::Development),
        "development" => Ok(Environment::Development),
        "test" => Ok(Environment::Test),
        "prod" => Ok(Environment::Production),
        "production" => Ok(Environment::Production),
        unknown => Err(anyhow!(r#"Unknown environment: "{}"!"#, unknown)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[derive(Deserialize, PartialEq, Debug)]
    pub struct Config {
        pub server: ServerConfig,
        pub database: DatabaseConfig,

        pub app_setting: String,
    }

    #[test]
    fn test_load_config_development() {
        figment::Jail::expect_with(|jail| {
            let config_dir = jail.create_dir("config")?;
            jail.create_file(
                config_dir.join("app.toml"),
                r#"
                app_setting = "Just a TOML App!"
            "#,
            )?;
            let environments_dir = jail.create_dir("config/environments")?;
            jail.create_file(
                environments_dir.join("development.toml"),
                r#"
                app_setting = "override!"
            "#,
            )?;

            jail.set_env("APP_SERVER__IP", "127.0.0.1");
            jail.set_env("APP_SERVER__PORT", "3000");
            jail.set_env(
                "APP_DATABASE__URL",
                "postgresql://user:pass@localhost:5432/my_app",
            );
            let config = load_config::<Config>(&Environment::Development).unwrap();

            assert_that!(
                config,
                eq(&Config {
                    server: ServerConfig {
                        ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                        port: 3000,
                    },
                    database: DatabaseConfig {
                        url: String::from("postgresql://user:pass@localhost:5432/my_app"),
                    },
                    app_setting: String::from("override!"),
                })
            );

            Ok(())
        });
    }

    #[test]
    fn test_load_config_test() {
        figment::Jail::expect_with(|jail| {
            let config_dir = jail.create_dir("config")?;
            jail.create_file(
                config_dir.join("app.toml"),
                r#"
                app_setting = "Just a TOML App!"
            "#,
            )?;
            let environments_dir = jail.create_dir("config/environments")?;
            jail.create_file(
                environments_dir.join("test.toml"),
                r#"
                app_setting = "override!"
            "#,
            )?;

            jail.set_env("APP_SERVER__IP", "127.0.0.1");
            jail.set_env("APP_SERVER__PORT", "3000");
            jail.set_env(
                "APP_DATABASE__URL",
                "postgresql://user:pass@localhost:5432/my_app",
            );
            let config = load_config::<Config>(&Environment::Test).unwrap();

            assert_that!(
                config,
                eq(&Config {
                    server: ServerConfig {
                        ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                        port: 3000,
                    },
                    database: DatabaseConfig {
                        url: String::from("postgresql://user:pass@localhost:5432/my_app"),
                    },
                    app_setting: String::from("override!"),
                })
            );

            Ok(())
        });
    }

    #[test]
    fn test_load_config_production() {
        figment::Jail::expect_with(|jail| {
            let config_dir = jail.create_dir("config")?;
            jail.create_file(
                config_dir.join("app.toml"),
                r#"
                app_setting = "Just a TOML App!"
            "#,
            )?;
            let environments_dir = jail.create_dir("config/environments")?;
            jail.create_file(
                environments_dir.join("production.toml"),
                r#"
                app_setting = "override!"
            "#,
            )?;

            jail.set_env("APP_SERVER__IP", "127.0.0.1");
            jail.set_env("APP_SERVER__PORT", "3000");
            jail.set_env(
                "APP_DATABASE__URL",
                "postgresql://user:pass@localhost:5432/my_app",
            );
            let config = load_config::<Config>(&Environment::Production).unwrap();

            assert_that!(
                config,
                eq(&Config {
                    server: ServerConfig {
                        ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                        port: 3000,
                    },
                    database: DatabaseConfig {
                        url: String::from("postgresql://user:pass@localhost:5432/my_app"),
                    },
                    app_setting: String::from("override!"),
                })
            );

            Ok(())
        });
    }
}
