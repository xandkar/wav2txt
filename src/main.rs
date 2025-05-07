use std::path::PathBuf;

use clap::Parser;
use tracing::level_filters::LevelFilter;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(short, long = "log", default_value_t = LevelFilter::DEBUG)]
    log_level: LevelFilter,

    #[clap(short, long)]
    model_file: PathBuf,

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
    let span = tracing::debug_span!(env!("CARGO_PKG_NAME"));
    let _span_guard = span.enter();
    tracing::info!(?cli, "Starting.");
    wav2txt::convert(
        &cli.input_file,
        cli.output_file.as_deref(),
        &cli.model_file,
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
