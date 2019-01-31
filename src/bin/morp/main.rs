use clap;

mod diff;
mod graph;

#[derive(Debug)]
enum Error {
    Graph(graph::Error),
    Diff(diff::Error),
    UnknownCommand(String),
}

fn main() -> Result<(), Error> {
    let matches = clap::App::new("morp")
        .subcommand(graph::get_args("graph"))
        .subcommand(diff::get_args("diff"))
        .get_matches();

    match matches.subcommand() {
        ("graph", Some(args)) => graph::run(args).map_err(Error::Graph),
        ("diff", Some(args)) => diff::run(args).map_err(Error::Diff),
        (c, _) => Err(Error::UnknownCommand(String::from(c))),
    }
}
