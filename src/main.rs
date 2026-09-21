mod fileops;

use std::cmp::min;
use std::collections::HashMap;
use regex::Regex;
use std::fs::File;
use std::io::{self, stdin, BufRead, BufReader, Read, Error, ErrorKind, Write};
use std::path::PathBuf;
use std::process;
use clap::Parser;
use rich_rs::{Column, Console, Table, Text};
use rich_rs::table::Row;
use crate::fileops::read_in_commands;

use enigo::{Enigo, InputResult, Keyboard, NewConError, Settings};

// q and Q deliberately omitted
static KEYS: [&str;60] = [ "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
                            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "r", "s", "t", "u", "v", "w", "x", "y", "z",
                            "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "R", "S", "T", "U", "V", "W", "X", "Y", "Z" ] ;

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

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

struct Histrionic {
    console: Console,
    enigo: Enigo,
    commands: Vec<String>,
    height : usize,
    page_offset: usize,
}

impl Histrionic {
    fn new(lines: Vec<String>) -> Self {

        let console = Console::new();
        let mut enigo = match Enigo::new(&Settings::default()) {
            Ok(enigo) => enigo,
            Err(_) => {
                eprintln!("failed to initialize terminal");
                process::exit(1);
            }
        } ;

        let height = min(console.height()-3, KEYS.len());

        Self {
            console,
            enigo,
            commands: lines,
            page_offset: 0,
            height,
        }
    }

    fn save_screen(&self) -> io::Result<()> {
        print!("\x1b[?1049h");
        io::stdout().flush()? ;
        Ok(())
    }

    fn restore_screen(&mut self) -> io::Result<()> {
        print!("\x1b[?1049l");
        io::stdout().flush()?;
        Ok(())
    }

    fn handle_page_down(&mut self, _: u8) -> bool {
        if( (self.page_offset+1) * self.height < self.commands.len() ) {
            self.page_offset += 1;
        }
        true
    }

    fn handle_page_up(&mut self, _: u8) -> bool {
        if( self.page_offset > 0) {
            self.page_offset -= 1 ;
        }
        true
    }


    fn render(&self) -> io::Result<Table> {
        let mut table = Table::new().with_show_header(false);
        let key_column = Column::new() ;
        let command_column = Column::new() ;

        table.add_column(key_column) ;
        table.add_column(command_column) ;

        let start = self.page_offset * self.height;
        let end = min((self.page_offset + 1) * self.height, self.commands.len());
        
        for (index, cmd) in self.commands[start..end].iter().enumerate() {
            let key = Text::from(KEYS[index]) ;
            let text = Text::from(cmd.as_str()) ;
            let row = Row::new(vec![Box::new(key), Box::new(text)]) ;
            table.add_row(row) ;
        }

        Ok(table)
    }

    fn send_text(&mut self, command: String, extracr:&str) -> io::Result<()> {
        match self.enigo.text(command.as_str()) {
            Ok(_) => (),
            _ =>  return Err(Error::new(ErrorKind::Other, "sending text failed"))
        } ;

        match self.enigo.text("\r") {
            Ok(_) => (),
            _ =>  return Err(Error::new(ErrorKind::Other, "sending text failed"))
        } ;

        match self.enigo.text(extracr) {
            Ok(_) => (),
            _ =>  return Err(Error::new(ErrorKind::Other, "sending text failed"))
        } ;


        Ok(())
    }

    fn main_loop(&mut self) -> io::Result<()> {
        type EscapeHandler = fn(&mut Histrionic, u8) -> bool ;

        /*
        let mut escape_handlers : HashMap<u8, EscapeHandler> = HashMap::new() ;
        escape_handlers.insert(b'6', Histrionic::handle_page_down);
        */

        let escape_handlers : HashMap<_,_> = [
            (54, Histrionic::handle_page_down as EscapeHandler),
            (53, Histrionic::handle_page_up),
            (b'\x1b', |_, _| false)// break
        ].into() ;


        let mut c : [u8;1] = [0] ;
        self.save_screen()? ;
        loop {
            self.console.clear()? ;
            let table = self.render()? ;

            self.console.print(&table, None, None, None, true, "")?;
            crossterm::terminal::enable_raw_mode()?;
            let _guard = RawModeGuard;

             stdin().read(&mut c)? ;
             let s = String::from_utf8_lossy(&c).to_string() ;
             let c = c[0] ;
             if c == b'q' || c == b'Q' {
                 break ;
             }
             if c == b'\x1b' {
                 let mut c : [u8;2] = [0;2] ;
                 stdin().read(&mut c)? ;
                 let c = c[1] ;

                 let f = match escape_handlers.get(&c) {
                     Some(handler) => handler,
                     None => continue, // unknown char
                 } ;
                 if !f(self, c) {
                     break ;
                 }
             }
            let key_index = match KEYS.iter().position(|k| k == &s) {
                Some(i) => i,
                None => continue,
            } ;
            let command = self.commands[self.height * self.page_offset + key_index].clone();
            self.send_text(command, "")?;
            break ;
        }
        self.restore_screen()? ;
        Ok(())
    }
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
