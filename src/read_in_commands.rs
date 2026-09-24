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
use regex::Regex;
use std::collections::HashSet;
use std::io::{self, BufRead};

/**
 *  Read the list of commands given by 'history' in bash or zsh
 *  Possibly apply a filtering regex
 *  Reverse the order so the most recent command will appear at the top
 *  Scrub out any duplicate commands for brevity.
 */
pub fn read_in_commands<R: BufRead>(
    reader: R,
    filter_regex: Option<&Regex>,
) -> io::Result<Vec<String>> {
    let remove_command_number_regex = match regex::Regex::new(r"^\s*([0-9]+\s+)") {
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
        let m = match remove_command_number_regex.captures(line.as_str())  {
            None => continue, // should always match, but...
            Some(m) => m
        } ;

        let len = m[0].chars().count() ;
        let command = line[len..].to_string() ; // strip off leading number from history
        commands.insert(0, command); // implicitly reverse order
    }

    // delete duplicate commands, preserving most recent
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
    if filter_regex.is_none() {
        commands.remove(0);
    } // pop off the first command which will be our invocation
    Ok(commands)
}