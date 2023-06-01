use clap::{Arg, ArgAction, Command};
use recsimu::gen;
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

            let mut config = gen::Config::new(PathBuf::from(input), PathBuf::from(output))?;

            config.build();
            config.generate()?;
        }
        _ => unreachable!(),
    };
    Ok(())
}
