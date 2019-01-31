use clap;
use std::iter::Flatten;
use std::path::Path;
use git2::{self, BranchType, DiffOptions, Repository};
use regex::Regex;
use std::collections::HashSet;
use std::path::PathBuf;
use petgraph::{
    Direction,
    Directed,
    algo,
    visit::{Visitable, Walker, Topo},
    graphmap::{
        DiGraphMap,
        NeighborsDirected
    }
};
use morp::monorepo::{self, Monorepo};

#[derive(Debug)]
pub enum Error {
    Monorepo(monorepo::MonorepoError)
}

#[derive(Debug)]
pub struct Options {
    path: Option<PathBuf>,
    prefix: Option<String>,
}

impl From<&clap::ArgMatches<'_>> for Options {
    fn from(args: &clap::ArgMatches) -> Self {
        Options {
            path: args.value_of("path").map(PathBuf::from),
            prefix: args.value_of("prefix").map(String::from),
        }
    }
}

pub fn get_args(name: &str) -> clap::App {
    clap::SubCommand::with_name(name)
        .version("0.1.0")
        .author("Elie Génard <elie.genard@protonmail.com>")
        .about("Get list of changed packages")
        .arg(
            clap::Arg::with_name("path")
                .long("path")
                .short("p")
                .takes_value(true)
                .help("Monorepo path"),
        )
        .arg(
            clap::Arg::with_name("prefix")
                .long("prefix")
                .takes_value(true)
                .help("Prefix added to changed packages output"),
        )
}

pub fn run(args: &clap::ArgMatches) -> Result<(), Error> {
    let options = Options::from(args);

    let path = options.path.unwrap_or(PathBuf::from("./"));

    let branch_name = "develop";

    let changed_files = get_changed_files(&path, branch_name);

    let changed_packages = get_changed_packages(changed_files);

    let monorepo = Monorepo::load(&path).map_err(Error::Monorepo)?;

    let graph = monorepo.get_deps_graph().map_err(Error::Monorepo)?;

    let changed_dep_packages = get_changed_dep_packages(changed_packages, &graph);

    let prefix = options.prefix.unwrap_or(String::from(""));

    changed_dep_packages.iter().for_each(|p| {
        println!("{}{}", prefix, p);
    });

    Ok(())
}

// Get a list of files changed between HEAD and a common ancestor with 'branch_name'
fn get_changed_files(path: &Path, branch_name: &str) -> Vec<Option<PathBuf>> {
    let repo = Repository::open(path).expect("Could not open repository.");

    let head = repo.head().expect("Could not get repo's HEAD ref.");
    let head_oid = head.target().expect("Could not get HEAD object id.");

    let branch = repo
        .find_branch(branch_name, BranchType::Local)
        .expect("Could not find branch.");
    let branch_oid = branch
        .into_reference()
        .target()
        .expect("Could not get branch object id.");

    // Find common ancestor of HEAD and 'branch'
    let base_oid = repo
        .merge_base(head_oid, branch_oid)
        .expect("Could not find common ancestor between branch and HEAD");

    let base_commit = repo
        .find_commit(base_oid)
        .expect("Could not find base commit.");

    let base_tree = base_commit
        .tree()
        .expect("Could not get tree from base commit.");

    let mut diff_options = DiffOptions::new();

    // Diff between common ancestor and HEAD
    let diff = repo
        .diff_tree_to_index(Some(&base_tree), None, Some(&mut diff_options))
        .expect("Could not get diff.");

    let mut changed_files: Vec<Option<PathBuf>> = Vec::new();

    diff.foreach(
        &mut |diff_delta, _| {
            let old_file = diff_delta.old_file();
            let new_file = diff_delta.new_file();

            changed_files.push(old_file.path().map(PathBuf::from));
            changed_files.push(new_file.path().map(PathBuf::from));

            true
        },
        None,
        None,
        None,
    )
    .expect("Error iterating over diff deltas.");

    changed_files
}

fn get_changed_packages(changed_files: Vec<Option<PathBuf>>) -> HashSet<String> {
    let package_re = Regex::new(r"packages/(.*?)/").unwrap();

    changed_files
        .into_iter()
        .filter_map(|p| p)
        .filter_map(|path| {
            package_re
                .captures(path.to_str().unwrap())
                // If a change is not in a package, it's at the root of the repo
                .map_or(Some(String::from("root")), |caps| {
                    caps.get(1).map(|cap| {
                        String::from(cap.as_str())
                    })
                })
        })
        .collect()
}

// TODO: Fix difference in names between package dir and name in package.json
fn get_changed_dep_packages(changed_packages: HashSet<String>, graph: &DiGraphMap<&str, f32>) -> HashSet<String> {
    assert!(!algo::is_cyclic_directed(graph));

    let mut packages = HashSet::new();

    changed_packages.into_iter().for_each(|p| {
        packages.insert(p.clone());

        let mut tovisit: Vec<&str> = vec![&p];

        while !tovisit.is_empty() {
            let next = tovisit.pop().unwrap();
            packages.insert(String::from(next));
            let mut nghbrs: Vec<&str> = graph.neighbors_directed(&next, Direction::Incoming).collect();
            tovisit.append(&mut nghbrs);
        }
    });

    packages
}
