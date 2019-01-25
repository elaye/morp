use clap::{App, Arg};
use std::path::PathBuf;

pub struct ProcessOptions {
    pub path: Option<PathBuf>,
}

pub fn parse() -> ProcessOptions {
    let matches = App::new("")
        .version("0.1.0")
        .author("Elie Génard <elie.genard@protonmail.com>")
        .about("Create depency graph for Lerna monorepo packages")
        .arg(
            Arg::with_name("path")
                .short("p")
                .takes_value(true)
                .help("Monorepo path"),
        )
        .get_matches();

    ProcessOptions {
        path: matches.value_of("path").map(PathBuf::from),
    }
}
