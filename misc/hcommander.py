# MIT License
#
# Copyright (c) 2026 Andrew Ellis Page
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
#
# SPDX short identifier: MIT
import os
import sys, re, argparse, tty
import termios
import subprocess
from io import StringIO

from rich.table import Table
from rich.text import Text
from rich.console import Console
from pynput import keyboard


class Histrionic(object):
    def __init__(self, debug=False):
        self._debug = debug
        self._escape_handlers = {}
        self._history = []
        self._page_offset = 0
        self._console = Console()
        self.setHandlers()
        self._highlight = -1
        return

    historyRE = re.compile(r"^[0-9]+\s+(.*)$")
    # q and Q deliberately omitted
    keys = "0123456789abcdefghijklmnoprstuvwxyzABCDEFGHIJKLMNOPRSTUVWXYZ"

    def setHandlers(self):
        self._escape_handlers['6'] = self._onPageDown
        self._escape_handlers['5'] = self._onPageUp
        self._escape_handlers['A'] = self._onHighlightUp
        self._escape_handlers['B'] = self._onHighlightDown
        return

    def _onHighlightDown(self, _c):
        self._highlight += 1
        if self._highlight >= len(self._history):
            self._highlight = len(self._history) - 1
        return

    def _onHighlightUp(self, _c):
        self._highlight -= 1
        if self._highlight < -1:
            self._highlight = -1
        return


    def _onPageDown(self, _c):
        self._page_offset += 1
        return
    def _onPageUp(self, _c):
        self._page_offset -= 1
        if self._page_offset < 0:
            self._page_offset = 0
        return

    def setHistory(self, path):
        seencmnds = set()
        self._history.clear()
        with open(path, "r") as f:
            lines = reversed(f.readlines()[0:-1]) # skip first command as it will always be our shortcut
            for line in map(str.strip, lines):
                m = self.historyRE.search(line)
                if not m:
                    continue
                cmd = m.group(1)
                if cmd in seencmnds:
                    continue
                seencmnds.add(cmd)
                self._history.append(cmd)

    def render(self):
        height = min(self._console.height - 3, len(self.keys))

        table = Table(show_header=False)
        table.add_column(width=1, justify="left")
        table.add_column()

        start = self._page_offset * height
        end = (self._page_offset + 1) * height
        for i, cmd in enumerate(self._history[start:end]):
            key = Text(self.keys[i])
            text = Text(cmd, no_wrap=True)
            if i + start == self._highlight:
                key.stylize("bold red")
                text.stylize("bold red")
            table.add_row(key, text)
        
        return table

    def mainLoop(self):
        console = self._console

        with console.screen():
            while True:
                console.clear()
                console.print(self.render())

                mode = tty.setraw(sys.stdin.fileno()) # save the mode
                try:
                    c = sys.stdin.read(1)
                    if c.encode() == b'\x1b': # escape
                        c = sys.stdin.read(1)
                        if c == '\x1b': # ESC-ESC
                            return # quit
                        if c != '[': # ???
                            continue
                        c = sys.stdin.read(1)
                        if c in self._escape_handlers:
                            if self._escape_handlers[c](c) == False : # None or True continue
                                return
                            continue
                        else:
                            if self._debug:
                                raise Exception(f"unrecognized escape sequence {c.encode()}")

                    if c == "\r":
                        index = self._highlight
                        extracr = "\n"
                    if c == 'q' or c == 'Q':
                        return # quit
                    else:
                        index = self.keys.find(c[0])
                        extracr = ""

                    if(index == -1):
                        continue # ???
                    height = min(self._console.height - 3, len(self.keys))

                    controller = keyboard.Controller()
                    if index + self._page_offset*height >= len(self._history):
                        continue
                    controller.type(self._history[index + self._page_offset*height] + "\n" + extracr)
                    break
                finally:
                    termios.tcsetattr(sys.stdin, termios.TCSANOW, mode) # restore the mode

        return


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-f", "--file")
    parser.add_argument("-d", "--debug", action="store_true")

    options = parser.parse_args()


    h = Histrionic(debug=options.debug)
    h.setHistory(options.file)
    h.mainLoop()


if __name__ == '__main__':
    main()