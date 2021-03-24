# Stock Tracking CLI

(Written for the Manning Live Project : [Building a Stock-Tracking CLI With Async Streams in Rust](https://www.manning.com/liveproject/building-a-stock-tracking-cli-with-async-streams-in-rust))

Implements a basic proof-of-concept CLI application for collecting stock market data.

## Installation

No external dependencies are required. This may be built by cloning the repository and running `cargo install --path .`. (Or run without
installing by, for example, `cargo run -- MSFT -p 2020-07-02`).

## Usage

Command line uasge may be checked with the `--help` option.

The stock tickers to be queried should be given on the command line, and a date for the period to start from given by `-p` option. 
Dates should be given in the form YYYY-MM-DD, and if not supplied be taken as sixty days previous to the current day.

For example `mng-tracker MSFT GOOG -p 2020-07-02` queries for the period starting 2nd July 2020, for MSFT and GOOG.

```
USAGE:
    mng-tracker [FLAGS] [OPTIONS] [tickers]...

FLAGS:
    -h, --help       Prints help information
        --stdout     Whether to write to stdout
    -V, --version    Prints version information

OPTIONS:
    -f, --file <from-file>    The (optional) file to take a csv list of tickers to track from
    -o, --out <out-file>      The (optional) file to write output
    -p, --period <period>     The period to use, expressed as 'yyyy-mm-dd' [default: ]

ARGS:
    <tickers>...    The tickers to process
```


An optional `-f` argument may be given to read a comma separated list of tickers from a file. e.g. `mng-tracker -f tickers.txt -p 2020-07-02`

Errors to stderr.
By default a web server is started on port 8081, and the last <n> values may be queried from `http://127.0.0.1:8081/tail/<n>`

For example:

`curl http://127.0.0.1:8081/tail/2` will return the last two tickers processed in json format.

## Example:

Running `mng-tracker MSFT GOOG AAPL UBER IBM -p 2020-07-02` produces:

    period start,symbol,price,change %,min,max,30d avg
    2020-07-02T13:30:00+00:00,MSFT,$233.84,0.14%,$199.41,$244.43,$237.07
    2020-07-02T13:30:00+00:00,GOOG,$2056.84,0.40%,$1415.21,$2128.31,$2066.86
    2020-07-02T13:30:00+00:00,AAPL,$123.79,0.37%,$90.57,$142.95,$127.97
    2020-07-02T13:30:00+00:00,UBER,$60.01,0.96%,$29.42,$63.18,$57.08
    2020-07-02T13:30:00+00:00,IBM,$128.30,0.12%,$103.74,$129.91,$121.81

## Technical

Quotes are taken from google finance, at 1d intervals. 
