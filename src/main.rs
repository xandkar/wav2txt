use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
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
    dbg!(&cli);
    wav2txt::convert(
        &cli.input_file,
        cli.output_file.as_deref(),
        &cli.model_file,
        cli.normalize,
    )?;
    Ok(())
}
