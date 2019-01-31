use clap;
use morp::monorepo::{self, Monorepo};
use petgraph::dot::{Config, Dot};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Monorepo(monorepo::MonorepoError),
    FileWrite(io::Error),
}

pub struct Options {
    pub path: Option<PathBuf>,
}

impl From<&clap::ArgMatches<'_>> for Options {
    fn from(args: &clap::ArgMatches) -> Self {
        Options {
            path: args.value_of("path").map(PathBuf::from),
        }
    }
}

pub fn get_args(name: &str) -> clap::App {
    clap::SubCommand::with_name(name)
        .version("0.1.0")
        .author("Elie Génard <elie.genard@protonmail.com>")
        .about("Create depency graph from Lerna monorepo packages")
        .arg(
            clap::Arg::with_name("path")
                .short("p")
                .takes_value(true)
                .help("Monorepo path"),
        )
}

pub fn run(args: &clap::ArgMatches) -> Result<(), Error> {
    let options = Options::from(args);

    let path = options.path.unwrap_or(PathBuf::from("./"));

    let monorepo = Monorepo::load(&path).map_err(Error::Monorepo)?;

    let graph = monorepo.get_deps_graph().map_err(Error::Monorepo)?;

    let graph_data = Dot::with_config(&graph, &[Config::EdgeNoLabel]);

    fs::write("dependencies.dot", format!("{:?}", graph_data)).map_err(Error::FileWrite)
}
