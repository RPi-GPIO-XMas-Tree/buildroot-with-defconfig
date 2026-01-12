#!/usr/bin/env python3

# ChatGPT is USELESS, I guess I'll do this myself...

import os.path
import sys
import time
import socket
import curses.ascii

DEF_SOCKET_PATH = "/tmp/si-tema2-qtest.sock"
GPIO_COUNT = 20


class QEMUQtestServer:
    """ Implements a qtest socket server. """

    def __init__(self, socket_path: str, force=False):
        self._sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        if os.path.exists(socket_path) and force:
            os.unlink(socket_path)
        self._sock.bind(socket_path)
        self._sock.listen(1)
        self._sockf = None

    @property
    def connected(self):
        return self._sockf != None

    def accept(self) -> None:
        """ Await connection from QEMU. """
        self._sock, _ = self._sock.accept()
        self._sockf = self._sock.makefile(mode='r')

    def cmd(self, qtest_cmd: str) -> str:
        """ Send a qtest command on the wire. """
        assert self._sockf is not None
        self._sock.sendall((qtest_cmd + "\n").encode('utf-8'))
        return self._sockf.readline()

    def close(self) -> None:
        """ Close this socket. """
        self._sock.close()
        if self._sockf:
            self._sockf.close()
            self._sockf = None

    def settimeout(self, timeout: float) -> None:
        """ Set a timeout, in seconds. """
        self._sock.settimeout(timeout)

class RPIGPIOManager():
    GPIO_RANGE=[0x3f200000, 0x3f201000]
    GPIO_SET_OFFSET=0x1c
    GPIO_RESET_OFFSET=0x28
    GPIO_READ_OFFSET=0x34

    def __init__(self, qtest_srv: QEMUQtestServer):
        self._qtest = qtest_srv

    def get_gpio_array(self, count):
        rreg = self.GPIO_RANGE[0] + self.GPIO_READ_OFFSET
        res = self._qtest_cmd(f"readl 0x{rreg:x}")
        if not res:  # unix socket not connected
            return None
        raw = int(res.split()[1], 0)
        gpio_state = []
        for idx in range(count):
            gpio_state.append(1 if (raw & (1 << idx)) else 0)
        return gpio_state

    def _qtest_cmd(self, cmd):
        return self._qtest.cmd(cmd)

class GPIOViewer():
    COLOR_ON = 1
    COLOR_OFF = 2
    COLOR_TITLE = 3

    ROWS = 2
    COLS = 10

    SLEEP = 0.2

    def __init__(self, gpio_mgr):
        self._gpio_mgr = gpio_mgr

    def run(self):
        """ Runs the CLI program """
        curses.wrapper(self._curses_loop)

    def _curses_loop(self, stdscr):
        self.stdscr = stdscr
        curses.curs_set(0)  # hide the cursor
        stdscr.nodelay(True)  # make key input non-blocking
        self._init_colors()
        # Main loop
        while True:
            self._curses_draw()
            if stdscr.getch() == curses.ascii.ctrl("c"):
                break
            time.sleep(0.1)

    def _init_colors(self):
        """ Initializes the color pairs for the program. """
        if not curses.has_colors(): return
        curses.start_color()
        curses.init_pair(self.COLOR_ON, curses.COLOR_BLACK, curses.COLOR_GREEN)
        curses.init_pair(self.COLOR_OFF, curses.COLOR_BLACK, curses.COLOR_RED)
        curses.init_pair(self.COLOR_TITLE, curses.COLOR_YELLOW, curses.COLOR_BLACK)

    def _curses_draw(self):
        """ Draws the screen. """
        height, width = self.stdscr.getmaxyx()
        
        required_lines = self.ROWS + 3
        required_cols = (self.COLS * 5) + 2
        if height < required_lines or width < required_cols:
            self.stdscr.clear()
            self.stdscr.addstr(0, 0, f"Screen too small! Min size: {required_lines}x{required_cols}")
            self.stdscr.refresh()
            return
        self.stdscr.clear()

        title = "QEMU GPIO Viewer"
        self.stdscr.addstr(1, 1, title, curses.color_pair(self.COLOR_TITLE) | curses.A_BOLD)

        gpio_state = self._gpio_mgr.get_gpio_array(GPIO_COUNT)
        for r in range(self.ROWS):
            for c in range(self.COLS):
                idx = r * self.COLS + c
                state = gpio_state[idx]
                text = f"[{idx: >2}]"
                color = self.COLOR_ON if state else self.COLOR_OFF
                self.stdscr.addstr(3 + r, 2 + (c * 5), text, curses.color_pair(color))

        self.stdscr.addstr(3 + self.ROWS + 2, 0, f"Use Ctrl+C to quit. Refresh rate: {self.SLEEP}s...", curses.A_DIM)
        self.stdscr.refresh()


if __name__ == '__main__':
    socket_path = DEF_SOCKET_PATH
    if len(sys.argv) > 1:
        socket_path = sys.argv[1]
    server = QEMUQtestServer(socket_path, force=True)
    gpio_mgr = RPIGPIOManager(server)
    print("Waiting for qemu to connect...")
    server.accept()

    cli = GPIOViewer(gpio_mgr)
    cli.run()

