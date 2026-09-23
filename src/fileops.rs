
use regex::Regex;
use std::collections::HashSet;
use std::io::{self, BufRead};

pub fn read_in_commands<R: BufRead>(
    reader: R,
    filter_regex: Option<&Regex>,
) -> io::Result<Vec<String>> {
    let re = match regex::Regex::new(r"^\s*([0-9]+\s+)") {
        Ok(re) => re,
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid regular expression")),
    } ;

    let mut commands = Vec::new();
    for line_result in reader.lines() {
        let line = line_result?;
        let should_include = match filter_regex {
            Some(re) => re.is_match(&line),
            None => true,
        };
        if ! should_include {
            continue;
        }

        // strip number
        let m = match re.captures(line.as_str())  {
            None => continue, // should always match, but...
            Some(m) => m
        } ;

        let len = m[0].chars().count() ;
        let command = line[len..].to_string() ; // strip off leading number from history
        commands.insert(0, command); // implicitly reverse order
    }

    // delete duplicate commands
    let mut seen : HashSet<String> = HashSet::new();
    let mut indices_to_delete : Vec<usize> = Vec::new();
    for (index, command) in commands.iter().enumerate() {
        if seen.contains(command) {
            indices_to_delete.insert(0, index);
        }
        seen.insert(command.clone());
    }

    // indices are in reverse order, so they remain valid
    // as they are deleted 'back to front'
    // i.e. delete i+1 keeps index i valid
    for index in indices_to_delete {
        commands.remove(index);
    }

    Ok(commands)
}

pub fn write_pid_file() -> io::Result<()> {
    let pid = std::process::id();
    std::fs::write("/run/hcommander.pid", pid.to_string())
}