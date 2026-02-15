![boom logo](https://github.com/user-attachments/assets/61e33a4d-c937-4451-aa81-a2c4cbad3b18)

<!--toc:start-->

- [Usage/Examples](#usageexamples)
  - [Common Examples](#common-examples)
  - [Config Validation](#config-validation)
  - [Bang Resolution](#bang-resolution)
- [Configuration](#configuration)
  - [Customising Bangs using External Sources](#customising-bangs-using-external-sources)
  - [Customising Bangs within the Config](#customising-bangs-within-the-config)
  - [Default Configuration](#default-configuration)
- [Run Locally](#run-locally)
- [Hosting](#hosting)
  - [Building/Installing](#buildinginstalling)
  - [Testing](#testing)
- [Acknowledgements](#acknowledgements)
- [License](#license)
<!--toc:end-->

## About

A lightweight and high-speed server for processing [DuckDuckGo Bangs](https://duckduckgo/bang.js).

#### Features

- **Bangs**
  - **DuckDuckGo-style bangs, imported from DuckDuckGo themselves**
  - **User-defined bangs with capability to override imported ones**
- **OpenSearch support**
- **Search suggestions**
  - **External Suggestions**
  - **Suggestions using Boom history (optional)**
- **Search history persistence (optional)**

<table>
<th>
  
Partly inspired by Theo's video on [Unduck](https://www.youtube.com/watch?v=_DnNzRaBWUU), `boom` provides the same, if not more, functionality.
Funnily enough, Theo recently made a video debating whether or not
tools should be rewritten in other languages - particularly Rust. Rust is the language that powers `boom`.

</th>

<th>

[![Video Thumbnail](http://img.youtube.com/vi/CvXsGWDozRw/0.jpg)](https://www.youtube.com/watch?v=CvXsGWDozRw "JavaScript is fast enough (don't rewrite your code)")

</th>
</table>

## Usage/Examples

**Using `boom` as a systemd service**

> [!NOTE]
> Config changes do not require a restart of `boom` (when changing existing bangs/adding new ones).
> Hot-reloading allows `boom` to configure bangs on-the-fly without a server restart. The only time a restart would
> be required is to:
>
> - Change the server address, port, or security (http vs https)
> - Remove bangs

```
[Unit]
Description=Boom Redirection Service (DuckDuckGo bangs to real search)
After=network.target

[Service]
ExecStart=%h/.cargo/bin/boom serve
Restart=on-failure
Environment=RUST_LOG=info

[Install]
WantedBy=default.target
```

### Common Examples

```bash
# Launch boom with the default port & address
# - (p)ort : 3000
# - (a)ddr: 127.0.0.1 [localhost]
boom serve

# Launch boom on port 3001
boom serve -p 3001

# Useful when launching boom at startup
# Waits for a valid internet connection instead of panicking
boom serve --await-internet

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

> [!IMPORTANT]
>
> On Linux, the `XDG_CONFIG_HOME` is used for the configuration home.
> In the case that this is not found, it will use `$HOME` as a fallback, similarly to MacOS.
>
> Windows relies on `%USERPROFILE%`.
>
> All OS will use the current working directory as their final choice. The tilde is automatically expanded within `bangs.source.filepath` to match these conditions.

### Customising Bangs using External Sources

Bangs can be imported from external sources, such as the [default provider (DuckDuckGo)](https://duckduckgo.com/bang.js). These sources
must be serialized in JSON with the following format:

```json
[
  {
    "s": "short name (e.g GitHub)",
    "t": "trigger (e.g gh)",
    "u": "url template (e.g https://github.com/{{{s}}}"
  }
]
```

These bangs are imported in a free-for-all fashion. There is no guaranteed order. Bangs imported from smaller sources with faster response times have a higher chance of being used, though not guaranteed due to the race-conditions taking place. This is not a design flaw, rather it ensures that mass amounts of sources can be imported in parallel.

> [!WARNING]
>
> If you need a bang definition to be consistently loaded, look at the next section.

### Customising Bangs within the Config

Bangs defined within the config have the highest precedence. They will never be overwritten by bangs from external sources.
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

The default configuration uses some, believe it or not, sane defaults to `boom`.

```toml
[server]
address = "127.0.0.1"
port = 3000
# Wait for the internet connection to be valid before attempting to serve
wait_for_internet = false
# Search suggestions url, with `{searchTerms}` as the template for any queries
# Suggestion endpoint must have a response structured as demonstrated within https://github.com/dewitt/opensearch/blob/master/opensearch-1-1-draft-6.md#opensearch-11-parameters
search_suggestions = "https://search.brave.com/api/suggest?q={searchTerms}"

[bangs]
# The entirety of `{{{s}}}` will be replaced with the search term
default_search_template = "https://google.com/search?q={{{s}}}"

# Set the path to a default bang file
[[bangs.source]]
# Whether to bother requesting the bangs or not
required = true
filepath = "~/.cache/boom/bangs.json"
# Where to fetch the bangs from
remote = "https://duckduckgo.com/bang.js"

# Existing bangs may be overwritten by their custom equivalent.
# Declaring a bang here is the only way to guarantee that it will be used and not overwritten
# in a race condition between sources.
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

## Hosting

If using a reverse proxy, ensure `boom` has access to the Host and X-Forwarded-Proto headers.
<br/>
For example, your nginx config may include something like:

```conf
location / {
  proxy_pass http://127.0.0.1:8080; # your boom instance here
  proxy_set_header host $host;
  proxy_set_header x-forwarded-proto $scheme
}
```

If `boom` doesn't have access to these, the OpenSearch functionality may not work as intended.

## Acknowledgements
The reason `boom` was ever created is due to the likes of two awesome people.\
Check out their implementations below:
- [Adolar0042](https://github.com/adolar0042) and his awesome [__redirector__](https://github.com/adolar0042/redirector)
- [rebelonion](https://github.com/rebelonion) and his insanely-fast C++ implementation of DDG Bangs - [BangServer](https://github.com/rebelonion/bangserver)

Of course, without the long exhaustive list of bangs created by [DuckDuckGo](https://duckduckgo.com/bang.js) themselves, and the idea, this would not exist.
## License

This project is licensed under [GPLv3.0](https://choosealicense.com/licenses/gpl-3.0/).
