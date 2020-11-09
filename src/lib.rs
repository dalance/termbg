use std::env;
use std::io::{self, Read};
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};
use timeout_readwrite::TimeoutReader;

/// Terminal
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Terminal {
    Rxvt,
    Screen,
    Tmux,
    Xterm,
}

/// 16bit RGB color
#[derive(Copy, Clone, Debug)]
pub struct Rgb {
    pub r: u16,
    pub g: u16,
    pub b: u16,
}

/// Background theme
#[derive(Copy, Clone, Debug)]
pub enum Theme {
    Light,
    Dark,
}

/// get detected termnial
pub fn terminal() -> Terminal {
    if env::var("COLORFGBG").is_ok() {
        Terminal::Rxvt
    } else if std::env::var("TMUX").is_ok() {
        Terminal::Tmux
    } else {
        let is_screen = if let Ok(term) = std::env::var("TERM") {
            term.starts_with("screen")
        } else {
            false
        };
        if is_screen {
            Terminal::Screen
        } else {
            Terminal::Xterm
        }
    }
}

/// get background color by `RGB`
pub fn rgb() -> Option<Rgb> {
    let term = terminal();
    if term == Terminal::Rxvt {
        from_rxvt()
    } else {
        from_xterm(term)
    }
}

/// get background color by `Theme`
pub fn theme() -> Option<Theme> {
    let rgb = rgb()?;

    // ITU-R BT.601
    let y = rgb.r as f64 * 0.299 + rgb.g as f64 * 0.587 + rgb.b as f64 * 0.114;

    if y > 32768.0 {
        Some(Theme::Light)
    } else {
        Some(Theme::Dark)
    }
}

fn from_rxvt() -> Option<Rgb> {
    let var = env::var("COLORFGBG").ok()?;
    let fgbg: Vec<_> = var.split(";").collect();
    let bg = u8::from_str_radix(fgbg[1], 10).ok()?;

    // rxvt default color table
    let (r, g, b) = match bg {
        // black
        0 => (0, 0, 0),
        // red
        1 => (205, 0, 0),
        // green
        2 => (0, 205, 0),
        // yellow
        3 => (205, 205, 0),
        // blue
        4 => (0, 0, 238),
        // magenta
        5 => (205, 0, 205),
        // cyan
        6 => (0, 205, 205),
        // white
        7 => (229, 229, 229),
        // bright black
        8 => (127, 127, 127),
        // bright red
        9 => (255, 0, 0),
        // bright green
        10 => (0, 255, 0),
        // bright yellow
        11 => (255, 255, 0),
        // bright blue
        12 => (92, 92, 255),
        // bright magenta
        13 => (255, 0, 255),
        // bright cyan
        14 => (0, 255, 255),
        // bright white
        15 => (255, 255, 255),
        _ => (0, 0, 0),
    };

    Some(Rgb {
        r: r * 256,
        g: g * 256,
        b: b * 256,
    })
}

fn from_xterm(term: Terminal) -> Option<Rgb> {
    // Query by XTerm control sequence
    if term == Terminal::Tmux {
        eprint!("\x1bPtmux;\x1b\x1b]11;?\x07\x1b\\\x03");
    } else if term == Terminal::Screen {
        eprint!("\x1bP\x1b]11;?\x07\x1b\\\x03");
    } else {
        eprint!("\x1b]11;?\x1b\\");
    }

    // Get query result
    let stdin = 0;
    let termios = Termios::from_fd(stdin).ok()?;
    let mut new_termios = termios.clone();
    new_termios.c_lflag &= !(ICANON | ECHO);
    tcsetattr(stdin, TCSANOW, &mut new_termios).ok()?;
    let reader = TimeoutReader::new(io::stdin(), std::time::Duration::from_millis(100));

    let mut buffer = [0; 25];
    let mut reader = reader.take(25);
    reader.read(&mut buffer).ok()?;
    tcsetattr(stdin, TCSANOW, &termios).ok()?;

    let mut start = None;
    let mut end = None;
    for (i, c) in buffer.iter().enumerate() {
        if *c == b':' {
            start = Some(i + 1);
        } else if *c == 0x1b && start.is_some() {
            end = Some(i);
        } else if *c == 0x7 && start.is_some() {
            end = Some(i);
        }
    }

    if let Some(start) = start {
        if let Some(end) = end {
            let s = String::from_utf8_lossy(&buffer[start..end]);
            let rgb: Vec<_> = s.split("/").collect();

            let r = decode_x11_color(rgb[0])?;
            let g = decode_x11_color(rgb[1])?;
            let b = decode_x11_color(rgb[2])?;
            Some(Rgb { r, g, b })
        } else {
            None
        }
    } else {
        None
    }
}

fn decode_x11_color(s: &str) -> Option<u16> {
    let len = s.len() as u32;
    let mut ret = u16::from_str_radix(s, 16).ok()?;
    ret *= 2u16.pow(4 - len);
    Some(ret)
}
