mod fileops;
mod histrionic;

use crate::fileops::read_in_commands;
use clap::Parser;
use regex::Regex;
use rich_rs::Console;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write, stdin};
use std::path::PathBuf;
use std::process;

use enigo::{Enigo, Keyboard};

#[derive(Parser, Debug, Clone)]
#[command(name = "hcommander", about = "Histrionic command selector")]
pub struct Args {
    /// File to read instead of stdin
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    /// Regular expression filter
    pub regex: Option<Regex>,
}

pub fn parse_args<I, T>(args: I) -> Result<Args, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    Args::try_parse_from(args)
}

struct Histrionic {
    console: Console,
    enigo: Enigo,
    commands: Vec<String>,
    height : usize,
    page_offset: usize,
}

fn main() {
    let args = Args::parse();

    let reader: Box<dyn BufRead> = match &args.file {
        Some(path) => {
            let file = match File::open(path) {
                Ok(f) => f,
                Err(err) => {
                    eprintln!("Error opening file '{}': {}", path.display(), err);
                    process::exit(1);
                }
            };
            Box::new(BufReader::new(file))
        }
        None => Box::new(stdin().lock()),
    };

    let lines = match read_in_commands(reader, args.regex.as_ref()) {
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        },
        Ok(lines) => {
            lines
        }
    } ;

    let mut histrionic = Histrionic::new(lines);

    match histrionic.main_loop() {
        Err(err) => {println!("Error: {}", err); process::exit(1);},
        Ok(_) => {process::exit(0)}
    }

}
