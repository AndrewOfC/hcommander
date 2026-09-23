use enigo::{Enigo, Keyboard, Settings};
use rich_rs::{Column, Console, Row, Table, Text};
use std::cmp::min;
use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read, Write};
use std::{io, process};
use std::fs::File;

// q and Q deliberately omitted
static KEYS: [&str;60] = [ "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
                            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "r", "s", "t", "u", "v", "w", "x", "y", "z",
                            "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "R", "S", "T", "U", "V", "W", "X", "Y", "Z" ] ;

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

pub struct Histrionic {
    console: Console,
    commands: Vec<String>,
    height : usize,
    page_offset: usize,
}


impl Histrionic {
    pub(crate) fn new(lines: Vec<String>) -> Self {
        let console = Console::new();
        let height = min(console.height()-3, KEYS.len());
        Self {
            console,
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
        if (self.page_offset+1) * self.height < self.commands.len()  {
            self.page_offset += 1;
        }
        true
    }

    fn handle_page_up(&mut self, _: u8) -> bool {
        if self.page_offset > 0 {
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

        let mut enigo = match Enigo::new(&Settings::default()) {
            Ok(enigo) => enigo,
            Err(_) => {
                return Err(Error::new(ErrorKind::Other, "Could not initialize enigo"))
            }
        } ;

        let command_string = format!("{}\r{}", command, extracr);
        match enigo.text(command_string.as_str()) {
            Ok(_) => (),
            _ =>  return Err(Error::new(ErrorKind::Other, "sending text failed"))
        } ;

        Ok(())
    }

    pub(crate) fn main_loop(&mut self) -> io::Result<()> {
        type KeyHandler = fn(&mut Histrionic, u8) -> bool ;

        let break_func = (|_, _| false) as KeyHandler;
        let key_handlers: HashMap<_,_> = [
            (0x36, Histrionic::handle_page_down as KeyHandler),
            (0x35, Histrionic::handle_page_up),
            (0x51, break_func), // Q
            (0x71, break_func), // q
        ].into() ;

        let mut c : [u8;1] = [0] ;
        let mut tty = File::open("/dev/tty")?;
        let mut command_option : Option<String> = None ;
        // self.save_screen()? ;
        loop {
            self.console.clear()? ;
            let table = self.render()? ;

            self.console.print(&table, None, None, None, true, "")?;
            crossterm::terminal::enable_raw_mode()?;
            let _guard = RawModeGuard;

             tty.read(&mut c)? ;
             let s = String::from_utf8_lossy(&c).to_string() ;
             let c = c[0] ;

             if key_handlers.contains_key(&c) {
                 let f = match key_handlers.get(&c) {
                     Some(handler) => handler,
                     None => continue, // unknown char
                 } ;
                 if !f(self, c) {
                     break ;
                 }
             }

             if c == b'\x1b' {
                 let mut c : [u8;2] = [0;2] ;
                 let _ = tty.read(&mut c)? ;
                 let c = c[1] ;

                 let f = match key_handlers.get(&c) {
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
            command_option = Some(self.commands[self.height * self.page_offset + key_index].clone());
            break ;
        }

        if let Some(command) = command_option {
            self.send_text(command, "")?;
        }

        //self.restore_screen()? ;
        Ok(())
    }
}