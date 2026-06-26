#[cfg(test)]
mod tests;
use crate::{error::AppError, mcp::tools::ToolCredentials, page::reader::ReaderCredentials};
use clap::Parser;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeOptions {
    pub credentials: ToolCredentials,
}
#[derive(Debug, Parser)]
#[command(version, about = "web MCP server")]
pub struct Cli {
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
        if credentials.exa_api_key.is_none() {
            return Err(AppError::config("--exa-api-key is required for stdio MCP."));
        }
        Ok(RuntimeOptions { credentials })
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
