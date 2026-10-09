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
use std::fs::OpenOptions;
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
use std::os::unix::process::parent_id;
use nix::ioctl_write_ptr ;
use nix::libc::uint8_t;
use crate::TextSender ;

ioctl_write_ptr!(send_to_tty, b'V', 0x5412, u8) ;

pub struct TTYSender {

}

impl TextSender for TTYSender {
    fn send_text(&self, text: String, extracr: &str) -> std::io::Result<()> {
        let ppid =parent_id();

        let ttypath = "/dev/pts/3".to_string();

        let foptions = OpenOptions::new().write(true).open(&ttypath)?;
        let fd : OwnedFd = foptions.into() ;
        println!("sending text to tty: {} to {}", text, ttypath);
        unsafe {
            match send_to_tty(fd.as_raw_fd(), text.as_ptr()) {
                Ok(_) => (),
                Err(e) => eprintln!("error {}", e)
            }
        }

        Ok(())
    }
}