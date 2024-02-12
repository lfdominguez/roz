use clap::Parser;
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, required = true, default_value_t, value_enum)]
    pub mode: Mode,

    #[arg(short, long, default_value = "-")]
    pub input: String,

    #[arg(long, default_value = "http://localhost")]
    pub ollama_url: String,

    #[arg(long, default_value = "codellama:7b")]
    pub ollama_model: String
}

#[derive(clap::ValueEnum, Debug, Clone, Serialize)]
pub enum Mode {
    GitCommit,
    GitDiff,
    Interactive
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Interactive
    }
}