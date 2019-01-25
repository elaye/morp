use monorepo_deps::monorepo::{self, Monorepo};
use petgraph::dot::{Config, Dot};
use std::fs;
use std::io;
use std::path::PathBuf;

mod options;

#[derive(Debug)]
enum Error {
    Monorepo(monorepo::MonorepoError),
    FileWrite(io::Error),
}

fn main() -> Result<(), Error> {
    let options = options::parse();

    let path = options.path.unwrap_or(PathBuf::from("packages/"));

    let monorepo = Monorepo::load(&path).map_err(Error::Monorepo)?;

    let graph = monorepo.get_deps_graph();

    let graph_data = Dot::with_config(&graph, &[Config::EdgeNoLabel]);

    fs::write("dependencies.dot", format!("{:?}", graph_data)).map_err(Error::FileWrite)
}
