aud2txt
=======

Audio to text tool, using [ggerganov](https://github.com/ggerganov)'s
[whisper.cpp](https://github.com/ggml-org/whisper.cpp) via
[whisper-rs](https://github.com/tazz4843/whisper-rs) and
[FFmpeg](https://en.wikipedia.org/wiki/FFmpeg).

install
-------

1. install FFmpeg (via your package manager or directly)
2. ensure `ffmpeg` command is available
3. `cargo install aud2txt`

usage
-----

### TL;DR

```txt
aud2txt <INPUT_FILE>
```

where `<INPUT_FILE>` is any media file readable by `ffmpeg`.

Also see the [demo](demo) script.

### options

```txt
Usage: aud2txt [OPTIONS] <INPUT_FILE>

Arguments:
  <INPUT_FILE>  Input audio file

Options:
  -l, --log <LOG_LEVEL>            [default: error]
  -m, --model-file <MODEL_FILE>
  -N, --no-normalize               Disable audio normalization before conversion to text
  -o, --output-file <OUTPUT_FILE>  Output text file
  -h, --help                       Print help
```

If `--model-file` argument is omitted, `aud2txt` will try to download and use
the default model from: <https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin>

If `--no-normalize` flag is passed, the normalization step will be skiped,
removing the runtime dependency on `ffmpeg`.
