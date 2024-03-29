use clap::{Arg, ArgAction, Command};
use recsimu::gen::config::Config;
use recsimu::sim::SimBuilder;
use recsimu::utils;
use std::error;
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn error::Error>> {
    let matches = Command::new("recsimu")
        .about("simulator for shape-changeable computer system")
        .version("0.1.0")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .author("Neccolini")
        // subcommands
        .subcommand(
            Command::new("gen")
                .about("generate a new simulation configuration")
                .arg(
                    Arg::new("input_file_path")
                        .short('i')
                        .long("input")
                        .action(ArgAction::Set)
                        .num_args(1)
                        .required(true),
                )
                .arg(
                    Arg::new("output_file_path")
                        .short('o')
                        .long("output")
                        .action(ArgAction::Set)
                        .num_args(1)
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("run")
                .about("run simulation")
                .arg(
                    Arg::new("input_file_path")
                        .short('i')
                        .long("input")
                        .action(ArgAction::Set)
                        .num_args(1)
                        .required(true),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .action(ArgAction::SetTrue)
                        .required(false),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("gen", gen_matches)) => {
            let input = gen_matches.get_one::<String>("input_file_path").unwrap();
            let default_output = format!("{input}.out.json");

            let output = if gen_matches.contains_id("output_file_path") {
                gen_matches.get_one::<String>("output_file_path").unwrap()
            } else {
                &default_output
            };

            let mut config = Config::new(PathBuf::from(input), PathBuf::from(output))?;

            config.build();
            config.generate()?;
        }
        Some(("run", run_matches)) => {
            let input = run_matches.get_one::<String>("input_file_path").unwrap();
            *utils::DEBUG_ENABLED.lock().unwrap() = run_matches.get_flag("verbose");
            let mut sim = SimBuilder::new(&PathBuf::from(input)).build()?;

            sim.run();
        }
        _ => unreachable!(),
    };
    Ok(())
}

// test
#[cfg(test)]
mod tests {
    use recsimu::{log::clear_log, network::vid::clear_vid_table};

    use super::*;
    use std::fs;

    #[test]
    fn test_run() {
        let testfile_dir = "tests/run/auto/";
        // testfile_dir以下のファイルを全てテストする
        for entry in fs::read_dir(testfile_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            dbg!(entry, path.clone());
            if path.is_file() {
                for _ in 0..5 {
                    clear_vid_table();
                    clear_log();

                    let mut sim = SimBuilder::new(&path).build().unwrap();
                    sim.run();

                    for node in sim.nodes.nodes.iter() {
                        assert_eq!(node.network.is_joined(), true, "{:?}", path.clone());
                    }
                }
            }
        }
    }
}
