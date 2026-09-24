// MIT License
//
// Copyright (c) 2026 Andrew Ellis Page
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// SPDX short identifier: MIT
mod read_in_commands;
mod histrionic;

use read_in_commands::read_in_commands;
use clap::Parser;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, stdin};
use std::path::PathBuf;
use std::process;

use histrionic::Histrionic;

#[derive(Parser, Debug, Clone)]
#[command(name = "hcommander", about = "Histrionic command selector")]
pub struct Args {
    /// File to read instead of stdin
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    /// Show version
    #[arg(short, long)]
    pub version: bool,


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


fn main() {
    let args = Args::parse();

    if args.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }


    let reader : Box<dyn BufRead> = if let Some(path) = &args.file {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(err) => {
                eprintln!("Error opening file '{}': {}", path.display(), err);
                process::exit(1);
            }
        };
        Box::new(BufReader::new(file))
    } else {
        Box::new(BufReader::new(stdin().lock()))
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
