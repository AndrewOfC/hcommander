


use std::fs::File;
use std::io::{self, BufReader};
use std::path::PathBuf;
use regex::Regex;

#[path = "../src/main.rs"]
mod main;
use main::{parse_args, run};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args_none() {
        let args = vec!["hcommander"];
        let parsed = parse_args(args).unwrap();
        assert!(parsed.file.is_none());
        assert!(parsed.regex.is_none());
    }

    #[test]
    fn test_parse_args_file_short() {
        let args = vec!["hcommander", "-f", "test.txt"];
        let parsed = parse_args(args).unwrap();
        assert_eq!(parsed.file, Some(PathBuf::from("test.txt")));
        assert!(parsed.regex.is_none());
    }

    #[test]
    fn test_parse_args_file_long() {
        let args = vec!["hcommander", "--file", "test.txt"];
        let parsed = parse_args(args).unwrap();
        assert_eq!(parsed.file, Some(PathBuf::from("test.txt")));
        assert!(parsed.regex.is_none());
    }

    #[test]
    fn test_parse_args_valid_regex() {
        let args = vec!["hcommander", r"^\d+"];
        let parsed = parse_args(args).unwrap();
        assert!(parsed.file.is_none());
        assert!(parsed.regex.is_some());
        assert!(parsed.regex.unwrap().is_match("123 test"));
    }

    #[test]
    fn test_parse_args_file_and_regex() {
        let args = vec!["hcommander", "-f", "test.txt", r"^\d+"];
        let parsed = parse_args(args).unwrap();
        assert_eq!(parsed.file, Some(PathBuf::from("test.txt")));
        assert!(parsed.regex.is_some());
        assert!(parsed.regex.unwrap().is_match("123 test"));
    }

    #[test]
    fn test_parse_args_invalid_regex() {
        let args = vec!["hcommander", "[unclosed"];
        let res = parse_args(args);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_args_too_many() {
        let args = vec!["hcommander", "foo", "bar"];
        let res = parse_args(args);
        assert!(res.is_err());
    }

    #[test]
    fn test_filter_and_reverse_no_regex() {
        let input = "first\nsecond\nthird\n";
        let lines = run(io::Cursor::new(input), None).unwrap();
        assert_eq!(lines, vec!["third", "second", "first"]);
    }

    #[test]
    fn test_filter_and_reverse_with_regex() {
        let input = "apple\nbanana\napricot\ncherry\n";
        let regex = Regex::new(r"^a").unwrap();
        let lines = run(io::Cursor::new(input), Some(&regex)).unwrap();
        assert_eq!(lines, vec!["apricot", "apple"]);
    }

    #[test]
    fn test_filter_and_reverse_empty_input() {
        let input = "";
        let lines = run(io::Cursor::new(input), None).unwrap();
        assert_eq!(lines, Vec::<String>::new());
    }

    #[test]
    fn test_run_from_sample_file() {
        let file = File::open("data/sample1.txt").unwrap();
        let lines = run(BufReader::new(file), None).unwrap();
        assert_eq!(
            lines,
            vec!["pushd", "popd", "pwd", "ls", "command 2", "command 1"]
        );
    }
}
