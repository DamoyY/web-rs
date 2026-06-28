#[cfg(test)]
mod tests;
use crate::{error::AppError, mcp::tools::ToolCredentials, page::reader::ReaderCredentials};
use clap::{Parser, ValueEnum};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeOptions {
    pub transport: Transport,
    pub credentials: ToolCredentials,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[non_exhaustive]
pub enum Transport {
    Http,
    Stdio,
}
#[derive(Debug, Parser)]
#[command(version, about = "web MCP server")]
pub struct Cli {
    #[arg(long, value_enum, default_value = "http")]
    transport: Transport,
    #[arg(long = "exa-api-key")]
    exa: Option<String>,
    #[arg(long = "jina-api-key")]
    jina: Option<String>,
    #[arg(long = "tinyfish-api-key")]
    tinyfish: Option<String>,
}
impl Cli {
    #[inline]
    #[must_use]
    pub fn from_env() -> Self {
        Self::parse()
    }
    #[inline]
    pub fn runtime_options(self) -> crate::Result<RuntimeOptions> {
        let credentials = self.credentials()?;
        Ok(RuntimeOptions {
            transport: self.transport,
            credentials,
        })
    }
    fn credentials(&self) -> crate::Result<ToolCredentials> {
        if self.jina.is_some() && self.tinyfish.is_some() {
            return Err(AppError::config(
                "Provide at most one remote reader API key: --jina-api-key or --tinyfish-api-key.",
            ));
        }
        Ok(ToolCredentials {
            exa_api_key: self.exa.clone(),
            reader: reader_credentials(self),
        })
    }
}
fn reader_credentials(cli: &Cli) -> Option<ReaderCredentials> {
    cli.jina
        .clone()
        .map(ReaderCredentials::Jina)
        .or_else(|| cli.tinyfish.clone().map(ReaderCredentials::TinyFish))
}
