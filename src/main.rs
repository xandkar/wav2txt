use std::{fs, io, path::PathBuf};

use anyhow::bail;
use clap::Parser;
use tracing::level_filters::LevelFilter;

macro_rules! crate_name {
    () => {
        env!("CARGO_PKG_NAME")
    };
}

#[derive(Parser, Debug)]
struct Cli {
    #[clap(short, long = "log", default_value_t = LevelFilter::DEBUG)]
    log_level: LevelFilter,

    #[clap(short, long)]
    model_file: Option<PathBuf>,

    /// Input audio file.
    #[clap(short, long)]
    input_file: PathBuf,

    /// Output text file.
    #[clap(short, long)]
    output_file: Option<PathBuf>,

    #[clap(short, long)]
    normalize: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    tracing_init(cli.log_level)?;
    let span = tracing::debug_span!(crate_name!());
    let _span_guard = span.enter();
    tracing::info!(?cli, "Starting.");
    let model_file = match cli.model_file {
        Some(path) => path,
        None => model_download()?,
    };
    wav2txt::convert(
        &cli.input_file,
        cli.output_file.as_deref(),
        &model_file,
        cli.normalize,
    )?;
    Ok(())
}

fn tracing_init(level: LevelFilter) -> anyhow::Result<()> {
    use tracing_subscriber::{
        fmt::{self, format::FmtSpan},
        layer::SubscriberExt,
        EnvFilter, Layer,
    };

    let span_events = if let Some(tracing::Level::TRACE) = level.into_level()
    {
        FmtSpan::NEW | FmtSpan::CLOSE
    } else {
        FmtSpan::CLOSE
    };

    let layer_stderr = fmt::Layer::new()
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_file(false)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_span_events(span_events)
        .with_filter(
            EnvFilter::from_default_env().add_directive(level.into()),
        );
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(layer_stderr),
    )?;
    Ok(())
}

fn model_download() -> anyhow::Result<PathBuf> {
    let cache_dir = match dirs::cache_dir() {
        Some(path) => path,
        None => {
            tracing::warn!(
                "No system cache directory found. Falling back on temporary."
            );
            tempfile::tempdir()?.into_path()
        }
    }
    .join(crate_name!());
    tracing::debug!(?cache_dir, "Determined cache directory.");
    let model_name = "base.en";
    let model_file_name = format!("ggml-{model_name}.bin");
    let model_file_path = cache_dir.join("models").join(&model_file_name);
    if !model_file_path.try_exists()? {
        if let Some(parent) = model_file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let model_url = format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{model_file_name}");
        tracing::info!(src = ?model_url, dst = ?model_file_path, "Model file not found. Downloading.");
        let resp = reqwest::blocking::get(model_url)?;
        let resp_status = resp.status();
        if !resp_status.is_success() {
            bail!(
                "Failed to download model file. Response status: {resp_status}"
            );
        }
        let mut dst = io::BufWriter::new(fs::File::create(&model_file_path)?);
        let mut src = resp;
        io::copy(&mut src, &mut dst)?;
    }
    Ok(model_file_path)
}
