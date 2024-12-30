pub mod cli;
pub mod error;
pub mod file_explorer;
pub mod parser;
pub mod printer;

use clap::Parser;
use cli::Args;
use error::MoleError;
use file_explorer::collect_files;
use hashbrown::HashMap;
use parser::data::FoundDependency;
use printer::pretty_table::print_table;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Args::parse();

    match explore(args) {
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
        Ok(deps) => {
            let parsed_deps = deps
                .into_iter()
                .map(|package| {
                    let res: [String; 3] =
                        [package.package_name, package.dep_version, package.path];
                    res
                })
                .collect();
            print_table(vec!["PACKAGE", "VERSION", "PATH"], parsed_deps);
            ExitCode::SUCCESS
        }
    }
}

fn explore(args: Args) -> Result<Vec<FoundDependency>, MoleError> {
    // Container for all explored files mathing the dependency
    let mut files = HashMap::new();

    collect_files(&args.path, &mut files, args.deep)?;
    parser::FileParser::new().parse(files, &args.name)
}
