use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
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

pub fn exec(cmd: &str, args: &[&str]) -> Result<Vec<u8>> {
    let out = std::process::Command::new(cmd).args(args).output()?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        dbg!(cmd);
        dbg!(args);
        eprintln!("{}", String::from_utf8(out.stderr.clone())?);
        Err(anyhow!("Failure in '{} {:?}'. out: {:?}", cmd, args, out))
    }
}

fn mktemp() -> Result<PathBuf> {
    let out = exec("mktemp", &[])?;
    let out = String::from_utf8(out)?;
    let out = out.trim(); // Output contains a trailing LF.
    Ok(PathBuf::from(out))
}

fn normalize(in_path: &Path) -> Result<PathBuf> {
    let out_path = mktemp()?;

    #[rustfmt::skip] // I mainly want each option-value pair on the same line.
    exec(
        "ffmpeg",
        &[
            // XXX Arg order matters!
            "-y", // Overwrite output file without prompting.
            "-i", in_path.as_os_str().to_str().ok_or_else(|| {
                anyhow!(
                    "Failed to convert input path to string: {:?}",
                    &in_path
                )
            })?,
            // Stream type "a", for "audio".
            // See: https://ffmpeg.org/ffmpeg.html#Stream-specifiers-1
            "-ar:a", "16000",        // 16KHz sampling frequency.
            "-ac:a", "1",            // 1 audio channel.
            "-codec:a", "pcm_s16le", // PCM signed 16-bit little-endian
            "-f", "wav",             // WAV format.
            out_path.as_os_str().to_str().ok_or_else(|| {
                anyhow!(
                    "Failed to convert output path to string: {:?}",
                    &out_path
                )
            })?,
        ],
    )?;

    Ok(out_path)
}

fn read_wav(path: &Path) -> Result<Vec<f32>> {
    let mut wav_reader = hound::WavReader::open(path)?;
    let spec = wav_reader.spec();
    dbg!(&spec);
    let hound::WavSpec {
        channels: ch,
        sample_rate: rate,
        bits_per_sample: bits,
        sample_format: fmt,
        ..
    } = spec;
    match rate {
        16000 => (),
        n => bail!(
            "Unsupported sample rate: {} Hz. Only 16000 Hz is supported.",
            n
        ),
    }
    let convert_stereo2mono = match ch {
        1 => false,
        2 => true,
        n => bail!("Unsupported number of channels: {}", n),
    };
    let convert_int2float = match (bits, fmt) {
        (16, hound::SampleFormat::Int) => true,
        (32, hound::SampleFormat::Float) => false,
        (bits, fmt) => bail!(
            "Unsupported combination of \
            bits ({}) and \
            format ({:?}) \
            in file: {:?}",
            bits,
            fmt,
            path
        ),
    };

    let mut samples: Vec<i32> = Vec::new();
    for sample_result in wav_reader.samples() {
        let sample = sample_result?;
        samples.push(sample);
    }
    let mut samples: Vec<f32> = if convert_int2float {
        let dat: Vec<i16> = samples.iter().map(|i| *i as i16).collect();
        whisper_rs::convert_integer_to_float_audio(&dat)
    } else {
        // TODO Does this dumb conversion make sense?
        samples.iter().map(|i| *i as f32).collect()
    };
    if convert_stereo2mono {
        samples = whisper_rs::convert_stereo_to_mono_audio(&samples)
            .map_err(|e| {
                anyhow!("failed to convert stereo to mono: {:?}", e)
            })?;
    }
    Ok(samples)
}

fn segments(data: &[f32], model: &Path) -> Result<Vec<String>> {
    let ctx = whisper_rs::WhisperContext::new(
        model.as_os_str().to_str().ok_or_else(|| {
            anyhow!("Failed to convert model path to &str: {:?}", model)
        })?,
    )
    .context("failed to load model")?;
    let params =
        whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy {
            best_of: 1,
        });
    let mut state = ctx.create_state().context("failed to create state")?;
    state.full(params, data).context("failed to run model")?;
    let num_segments = state
        .full_n_segments()
        .context("failed to get number of segments")?;
    let mut text_segments = Vec::new();
    for i in 0..num_segments {
        // Not .collect()ing so that we can short-circuit on errors.
        text_segments.push(
            state
                .full_get_segment_text(i)
                .context("failed to get segment")?,
        );
    }
    Ok(text_segments)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    dbg!(&cli);

    let audio_file = if cli.normalize {
        normalize(&cli.audio_file)?
    } else {
        cli.audio_file
    };
    let audio_data = read_wav(audio_file.as_path())?;
    let text_segments = segments(&audio_data, cli.model_file.as_path())?;

    let mut text_buf: Box<dyn std::io::Write> = match cli.text_file {
        None => Box::new(std::io::stdout().lock()),
        Some(ref path) => {
            let buf = std::fs::File::create(path)?;
            Box::new(buf)
        }
    };
    for text in text_segments {
        writeln!(text_buf, "{}", &text)?;
    }
    Ok(())
}
