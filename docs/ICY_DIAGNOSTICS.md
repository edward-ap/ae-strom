# icy_peek — CMD metadata inspector

`icy_peek` inspects a radio server's ICY metadata without playback, an audio
device, a window or a player profile. Ported from mini-radio, it uses Strøm's
cancellable HTTP transport and ICY parser. Temporary audio/timing test clients
have been removed; the player and this metadata utility are the remaining bins.

## Build and run

```text
cargo build --release --locked --bin icy_peek
target\release\icy_peek.exe "https://example.org/radio"
```

For portable use, copy `target/release/icy_peek.exe` beside `icy_peek.cmd` and
double-click the launcher, or pass a URL to either. The default stream is
ROCK ANTENNE Deutschrock:
`https://s2-webradio.rockantenne.de/deutschrock/stream/aacp`.

```text
icy_peek.cmd "https://example.org/radio" --seconds 120 --log "C:\Temp\capture.jsonl"
icy_peek.exe --help
```

The optional URL is positional or supplied with `--url`, once only. `--seconds`
sets a positive capture duration. Without it, closing standard input ends the
capture. `--log` selects a new JSONL file; existing files are never overwritten.
By default logs are created in `local-data/icy-peek-<time>-<pid>.jsonl` beside
the executable. The launcher enables UTF-8 console output.

## Interactive commands

Type a command and press Enter:

| Command | Behavior |
| --- | --- |
| `m [note]` | Record a note with the current observer title. |
| `p` | Open a fresh ICY connection, leaving the observer running; close it after its first title or ten seconds. |
| `r` | Stop the observer and probe, then restart the observer. |
| `q` | Cancel and join HTTP/metadata workers, flush the log and exit. |

Console and JSONL output include HTTP headers, request/session identifiers, raw
ICY blocks, normalized title/station changes, connection errors and reconnects.
JSONL records carry wall-clock and elapsed timestamps. Each record is flushed;
a log write failure stops the capture. Logs may contain URLs, session headers
and personal notes, so `local-data` and logs stay outside Git.

## Interpreting results

The inspector reads its own connection, not the audio player's connection. Two
connections to the same URL can have different session positions. A fresh probe
helps compare sessions but cannot establish what the player currently hears.
Reconnects also do not prove the server deliberately disconnected a client;
inspect the reported EOF/error/timeout reason.

The player now prioritizes ICY from its audio connection, then container tags
when ICY is unavailable, then configured/discovered metadata fallback streams.
Separate fallback streams have no universal synchronization guarantee. Playback
and HTTP timing instrumentation remains available in the library for loopback
tests; no standalone audio diagnostic client is shipped.

## Automated checks

```text
cargo test --release --locked --bin icy_peek --test icy_console
```

Synthetic loopback tests cover fragmented ICY blocks, header/title logging,
EOF reconnects, notes, fresh probes, bounded shutdown and zero remaining workers.
They do not access a public station or open an audio device.
