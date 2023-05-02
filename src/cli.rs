use clap::{App, Arg};

pub fn cli_interface() -> clap::ArgMatches {
    let matches = App::new("Open LLM Server")
        .version("1.0")
        .about("Expose and run local LLMs via HTTP API using a single command.")
        .subcommand(
            App::new("run")
                .about("Run the app")
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
                ),
        )
        .subcommand(App::new("help").about("Prints help information"))
        .get_matches();
    return matches;
}
