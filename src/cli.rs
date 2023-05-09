use crate::APP_VERSION;
use clap::{App, Arg};

pub fn cli_interface() -> clap::ArgMatches {
    let matches = App::new("Open LLM Server")
        .version(APP_VERSION)
        .about("Expose and run local LLMs via HTTP API using a single command.")
        .subcommand(
            App::new("run")
                .about("Load the LLM and start the webserver")
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .takes_value(true)
                        .help("The port on which to run the server"),
                )
                .arg(
                    Arg::new("model")
                        .short('m')
                        .long("model")
                        .takes_value(true)
                        .help("The path to the local LLM model file"),
                )
                .arg(
                    Arg::new("temp")
                        .short('t')
                        .long("temp")
                        .takes_value(true)
                        .help("The sampling temperature the LLM should use (Default: 0.7)"),
                )
                .arg(
                    Arg::new("freq_penalty")
                        .short('f')
                        .long("freq_penalty")
                        .takes_value(true)
                        .help("The frequency(repeat) penalty the LLM should use (Default: 1.2)"),
                )
                .arg(
                    Arg::new("output_tokens")
                        .short('o')
                        .long("output_tokens")
                        .takes_value(true)
                        .help("The max number of output tokens you want the model to return (Default: 2048)"),
                )
                .arg(
                    Arg::new("num_threads")
                        .short('n')
                        .long("num_threads")
                        .takes_value(true)
                        .help("Number of threads the LLM should use (Default: 8)"),
                )
                .arg(
                    Arg::new("api_key")
                        .short('a')
                        .long("api-key")
                        .takes_value(true)
                        .help("The API key to protect the server"),
                ),
        )
        .subcommand(App::new("help").about("Prints help information"))
        .get_matches();
    return matches;
}
