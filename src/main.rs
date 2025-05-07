use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(long = "in-model", short = 'm')]
    model_file: PathBuf,

    #[clap(long = "in-audio", short = 'a')]
    audio_file: PathBuf,

    #[clap(long = "out-text", short = 'o')]
    text_file: Option<PathBuf>,

    #[clap(long, short, default_value_t = false)]
    normalize: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    dbg!(&cli);
    wav2txt::convert(
        &cli.audio_file,
        cli.text_file.as_deref(),
        &cli.model_file,
        cli.normalize,
    )?;
    Ok(())
}
