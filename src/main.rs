use std::fmt::format;
use clap::Parser;
use std::io::{ErrorKind, Read};
use git2::DiffFormat;
use git2::string_array::StringArray;
use ollama_rs::{
    generation::completion::{
        request::GenerationRequest, GenerationContext, GenerationResponseStream,
    },
    Ollama
};
use tokio_stream::StreamExt;
use tokio::io::{stdout, AsyncWriteExt};

mod args;

#[tokio::main]
async fn main() -> Result<(), std::io::Error>
{
    let args = args::Args::parse();

    let ollama = Ollama::new(args.ollama_url, 11434);

    ollama.show_model_info(args.ollama_model.clone()).await.map_err(|err| {
        std::io::Error::new(ErrorKind::NotConnected, err)
    })?;

    let request = match args.mode {
        args::Mode::GitCommit => {
            let repo = match git2::Repository::open("./") {
                Ok(repo) => repo,
                Err(e) => panic!("Failed to open git repository: {}", e),
            };

            GenerationRequest::new(args.ollama_model, String::new()).system("".to_string())
        }
        args::Mode::GitDiff => {
            let repo = match git2::Repository::open("./") {
                Ok(repo) => repo,
                Err(e) => panic!("Failed to open git repository: {}", e),
            };

            let diff = match repo.diff_index_to_workdir(None, None) {
                Ok(diff) => diff,
                Err(e) => panic!("Failed to get git diff: {}", e),
            };

            let mut diff_lines: Vec<String> = Vec::new();

            diff.print(DiffFormat::Patch, |_, hunk, diff_line| {
                let line = String::from_utf8(diff_line.content().to_vec()).unwrap_or(String::new());

                if hunk.is_some() && diff_line.origin() != 'H' {
                    diff_lines.push(format!("{}{}", diff_line.origin(), line));
                } else {
                    diff_lines.push(line);
                }

                true
            }).map_err(|err| {
                std::io::Error::new(ErrorKind::BrokenPipe, err)
            })?;

            GenerationRequest::new(args.ollama_model, diff_lines.join("")).system("".to_string())
        }
        args::Mode::Interactive => {
            let input = if args.input == "-" {
                let stdin = std::io::stdin();
                let mut handle = stdin.lock();

                let mut std_input = Vec::new();

                handle.read_to_end(&mut std_input)?;

                String::from_utf8(std_input).map_err(|err| {
                    std::io::Error::new(ErrorKind::BrokenPipe, err)
                })?
            } else {
                args.input.to_string()
            };

            GenerationRequest::new(args.ollama_model, input).system("You're a CLI interface that ask something and respond".to_string())
        }
    };

    let mut stream: GenerationResponseStream = ollama.generate_stream(request).await.map_err(|err| {
        std::io::Error::new(ErrorKind::BrokenPipe, err)
    })?;

    let mut stdout = tokio::io::stdout();

    while let Some(Ok(res)) = stream.next().await {
        for ele in res {
            stdout.write_all(ele.response.as_bytes()).await?;
            stdout.flush().await?;
        }
    }

    Ok(())
}