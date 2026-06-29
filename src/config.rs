use std::{
    env,
    net::{AddrParseError, SocketAddr},
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
    pub resources: ResourcePaths,
}

#[derive(Debug, Clone)]
pub struct ResourcePaths {
    pub normalization: PathBuf,
    pub mcc_risk: PathBuf,
    pub references: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, AddrParseError> {
        Ok(Self {
            bind_addr: bind_addr_from_env()?,
            resources: ResourcePaths {
                normalization: path_from_env(
                    "RINHA_NORMALIZATION_PATH",
                    "resources/normalization.json",
                ),
                mcc_risk: path_from_env("RINHA_MCC_RISK_PATH", "resources/mcc_risk.json"),
                references: path_from_env(
                    "RINHA_REFERENCES_PATH",
                    "resources/example-references.json",
                ),
            },
        })
    }
}

fn bind_addr_from_env() -> Result<SocketAddr, AddrParseError> {
    if let Ok(bind_addr) = env::var("BIND_ADDR") {
        return bind_addr.parse();
    }

    let port = env::var("PORT").unwrap_or_else(|_| "9999".to_string());
    format!("0.0.0.0:{port}").parse()
}

fn path_from_env(variable: &str, default: &str) -> PathBuf {
    env::var(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(default))
}
