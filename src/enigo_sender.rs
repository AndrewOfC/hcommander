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
use std::{io, thread};
use std::io::Error;
use std::time::Duration;
use enigo::{Enigo, Keyboard, Settings};
use crate::textsender::TextSender;

pub struct EnigoSender {

}

impl TextSender for EnigoSender {
    fn send_text(&self, command: String, extracr:&str) -> io::Result<()> {
        thread::sleep(Duration::from_millis(150)); // delay so the key release event doesn't 'eat' our output
        let mut enigo = match Enigo::new(&Settings::default()) {
            Ok(enigo) => enigo,
            Err(e) => {
                return Err(Error::other(format!("Could not initialize enigo: {e}")))
            }
        } ;

        let command_string = format!("{}\r{}", command, extracr);
        match enigo.text(command_string.as_str()) {
            Ok(_) => (),
            _ =>  return Err(Error::other("sending text failed"))
        } ;

        Ok(())
    }
}