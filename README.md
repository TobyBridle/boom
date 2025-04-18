
<img src="https://github.com/user-attachments/assets/61e33a4d-c937-4451-aa81-a2c4cbad3b18" width="900" height="400" />

# Boom

A lightweight and high-speed server for processing [DuckDuckGo Bangs](https://duckduckgo/bang.js).

## Usage/Examples

```bash
# Launch boom with the default port & address
# - (p)ort : 3000
# - (a)ddr: 127.0.0.1 [localhost]
boom serve

# Launch boom on port 3001
boom serve -p 3001

# Launch boom using a temporary config
boom -c test_config.toml serve
```

### Config validation
```bash
# Validates the default config found at ~/.config/boom/config.toml (auto-created if does not exist)
boom validate

boom validate -c <path-to-config>
```

### Bang resolution
*Note: boom does not need to be running prior to this - all bangs will be fetched and cached.*
```bash
#            ⌄ note the very intentional usage of the single-quote
boom resolve '!boom'
#            ^ to prevent shell globbing

# Resolve using a non-default config
boom -c <path-to-custom-config> resolve
```

Do note that just because `boom resolve` resolves properly, your server may not. This occurs
in cases in which the server is using an out-of-date version of the config, or a different config entirely.\
If you suspect this to be the case, just restart `boom`.

## Configuration
A default configuration file can be found at `~/.config/boom/config.toml`\
This is automatically created when `boom` cannot find a config file and is used\
unless specified otherwise (via the `-c` tac)

Please note that functionality similar to that of a URL-shortener can be achieved by
omitting any search template.
```toml
[bangs.custom.shortened]
trigger = "sh"
template = "https://mysuperlongurl.com?with_some_params=1234"
#           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
#           note: the lack of a search template {{{s}}}
```

It's recommended to try and `resolve` (`boom resolve <query>`) any custom bangs\
after they've been added to the config.
Using the example above:
```bash
[tobybridle:$] boom resolve '!sh'
Resolved: "https://mysuperlongurl.com?with_some_params=1234"
```


### Default Configuration
The default configuration some, believe it or not, sane defaults to `boom`.
```toml
[server]
address = "127.0.0.1"
port = 3000

[bangs]
# The entirety of `{{{s}}}` will be replaced with the search term
default_search_template = "https://google.com/search?q={{{s}}}"
#                         "https://www.bing.com/search?q={{{s}}}"
#                         "https://www.qwant.com/?l=en&q={{{s}}}"

# Set the path to a default bang file
[bangs.default]
# Whether to bother requesting the bangs or not
enabled = true
filepath = "~/.cache/boom/bangs.json"
# Where to fetch the bangs from
remote = "https://duckduckgo.com/bang.js"

[bangs.custom]
boomdev = { template = "https://github.com/tobybridle/boom", trigger = "boomdev" }
# ^ shortname

# You can also set them like this
[bangs.custom.amazingdev]
# `!amazedev boom` resolves to the url for this project!
template = "https://github.com/tobybridle/{{{s}}}"
trigger = "amazedev"
```
## Run Locally

Clone the project

```bash
  git clone https://github.com/tobybridle/boom
```

Go to the project directory

```bash
  cd boom
```

### Building/Installing
```bash
# Typical release build
cargo build --release

# Release build with SIMD features toggled on (system support required)
RUSTFLAGS="-C target-feature=+avx2 -Zcrate-attr=feature(stdarch_x86_avx512)" cargo build --release

# Optionally, you can install `boom` via:
cargo install --path .
```

### Testing
```bash
# Similarly to building:
cargo test --release

# SIMD
RUSTFLAGS="-C target-feature=+avx2 -Zcrate-attr=feature(stdarch_x86_avx512)" cargo test --release

# Benchmarks
cargo bench

# Benchmarks w/ SIMD
RUSTFLAGS="-C target-feature=+avx2 -Zcrate-attr=feature(stdarch_x86_avx512)" cargo bench
```
## Acknowledgements
The reason `boom` was ever created is due to the likes of two awesome people.\
Check out their implementations below:
- [Adolar0042](https://github.com/adolar0042) and his awesome [__redirector__](https://github.com/adolar0042/redirector)
- [rebelonion](https://github.com/rebelonion) and his insanely-fast C++ implementation of DDG Bangs - [BangServer](https://github.com/rebelonion/bangserver)
Of course, without the long exhaustive list of bangs created by [DuckDuckGo](https://duckduckgo.com/bang.js) themselves, and the idea, this would not exist.
## License

This project is licensed under [GPLv3.0](https://choosealicense.com/licenses/gpl-3.0/).
