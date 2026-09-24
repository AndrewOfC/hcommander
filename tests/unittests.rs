


use std::io::{self, BufReader};
use std::path::PathBuf;

#[path = "../src/main.rs"]
mod main;
use main::{parse_args};

#[path = "../src/read_in_commands.rs"]
mod read_in_commands ;
use read_in_commands::read_in_commands;

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Error, Seek, SeekFrom, Write};
    use std::num::ParseIntError;
    use regex::Regex;
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


    const sample : &[u8] = br#"        1 6 ls a
    2 5 ls b
    1 4 ls a
    3 3 ls c
    4 2 git 1
    5 1 git a
    6 a
"# ;

    fn gen_sample() -> io::Result<Cursor<Vec<u8>>>  {
        let mut memfile = Cursor::new(Vec::<u8>::new());
        memfile.write_all(sample)? ;

        memfile.seek(SeekFrom::Start(0))?;
        Ok(memfile)
    }

    fn intparse(s: &str) -> io::Result<usize> {
        match s.parse::<usize>() {
            Ok(i) => { Ok(i) },
            Err(_) => { return Err(Error::other("failed to parse")) }
        }
    }

    #[test]
    fn test_reading_commands() -> io::Result<()> {
        let mut memfile = gen_sample()?;

        let commands = read_in_commands(memfile, None)?;
        assert_eq!(commands.len(), 6);

        for (i, command) in commands.iter().enumerate() {
            let s2 = &command[0..1] ;
            let j = intparse(s2)? ;
            assert_eq!(j, i+1) ;
        } // for

        Ok(())
    }

    #[test]
    fn test_reading_command_w_filter() -> io::Result<()> {
        let mut memfile = gen_sample()?;

        let r = match Regex::new("git") {
            Ok(r) => r,
            Err(_) => { return Err(Error::other("failed to parse")) }
        } ;
        let commands = read_in_commands(memfile, Some(&r))?;
        assert_eq!(commands.len(), 2);

        for (i, command) in commands.iter().enumerate() {
            let s2 = &command[0..1] ;
            let j = intparse(s2)? ;
            assert_eq!(j, i+1) ;
        } // for

        Ok(())

    }
}
