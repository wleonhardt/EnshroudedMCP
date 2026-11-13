//! Server endpoint configuration over the shared loopback transport.

pub(crate) use mcp_transport::Config;

pub(crate) fn config() -> Config {
    Config::from_env("ENSHROUDED_MCP_PORT", "ENSHROUDED_MCP_TOKEN", 8751)
}

pub(crate) fn run(config: &Config, tools: &impl policy::mcp::Tools) {
    mcp_transport::run(config, tools, env!("CARGO_PKG_VERSION"));
}
