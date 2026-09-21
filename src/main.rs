
use regex::Regex;
use std::env;
use std::io::{self, BufRead, Cursor, Seek, Write};
use std::process;
use rich_rs::Console;

struct Histrionic {
    console: Console
}

impl Histrionic {
    fn new() -> Self {

        let console = Console::new();

        Self {
            console
        }
    }
}

pub fn parse_regex_arg<I: IntoIterator<Item = String>>(args: I) -> Result<Option<Regex>, String> {
    let args_vec: Vec<String> = args.into_iter().collect();
    match args_vec.len() {
        1 => Ok(None),
        2 => Regex::new(&args_vec[1])
            .map(Some)
            .map_err(|e| format!("Invalid regular expression '{}': {}", args_vec[1], e)),
        _ => {
            let prog_name = args_vec.first().map(|s| s.as_str()).unwrap_or("hcommander");
            Err(format!("Usage: {} [regex]", prog_name))
        }
    }
}

pub fn filter_to_memory_file<R: BufRead>(
    reader: R,
    regex: Option<&Regex>,
) -> io::Result<Cursor<Vec<u8>>> {
    let mut mem_file = Cursor::new(Vec::new());
    for line_result in reader.lines() {
        let line = line_result?;
        let should_include = match regex {
            Some(re) => re.is_match(&line),
            None => true,
        };
        if should_include {
            writeln!(mem_file, "{}", line)?;
        }
    }
    Ok(mem_file)
}

pub fn reverse_memory_file_lines(mem_file: &mut Cursor<Vec<u8>>) -> io::Result<Vec<String>> {
    mem_file.rewind()?;
    let mut lines = Vec::new();
    for line in mem_file.lines() {
        lines.push(line?);
    }
    lines.reverse();
    Ok(lines)
}

pub fn run<R: BufRead, W: Write>(
    reader: R,
    mut writer: W,
    regex: Option<&Regex>,
) -> io::Result<()> {
    let mut mem_file = filter_to_memory_file(reader, regex)?;
    let reversed_lines = reverse_memory_file_lines(&mut mem_file)?;
    for line in reversed_lines {
        writeln!(writer, "{}", line)?;
    }
    Ok(())
}

fn main() {
    let regex = match parse_regex_arg(env::args()) {
        Ok(r) => r,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    let stdin = io::stdin();
    let stdout = io::stdout();
    if let Err(e) = run(stdin.lock(), stdout.lock(), regex.as_ref()) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args_none() {
        let args = vec!["hcommander".to_string()];
        let res = parse_regex_arg(args).unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn test_parse_args_valid_regex() {
        let args = vec!["hcommander".to_string(), r"^\d+".to_string()];
        let res = parse_regex_arg(args).unwrap();
        assert!(res.is_some());
        assert!(res.unwrap().is_match("123 test"));
    }

    #[test]
    fn test_parse_args_invalid_regex() {
        let args = vec!["hcommander".to_string(), "[unclosed".to_string()];
        let res = parse_regex_arg(args);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_args_too_many() {
        let args = vec![
            "hcommander".to_string(),
            "foo".to_string(),
            "bar".to_string(),
        ];
        let res = parse_regex_arg(args);
        assert!(res.is_err());
    }

    #[test]
    fn test_filter_and_reverse_no_regex() {
        let input = "first\nsecond\nthird\n";
        let mut output = Vec::new();
        run(io::Cursor::new(input), &mut output, None).unwrap();
        let result_str = String::from_utf8(output).unwrap();
        assert_eq!(result_str, "third\nsecond\nfirst\n");
    }

    #[test]
    fn test_filter_and_reverse_with_regex() {
        let input = "apple\nbanana\napricot\ncherry\n";
        let regex = Regex::new(r"^a").unwrap();
        let mut output = Vec::new();
        run(io::Cursor::new(input), &mut output, Some(&regex)).unwrap();
        let result_str = String::from_utf8(output).unwrap();
        assert_eq!(result_str, "apricot\napple\n");
    }

    #[test]
    fn test_filter_and_reverse_empty_input() {
        let input = "";
        let mut output = Vec::new();
        run(io::Cursor::new(input), &mut output, None).unwrap();
        let result_str = String::from_utf8(output).unwrap();
        assert_eq!(result_str, "");
    }
}
