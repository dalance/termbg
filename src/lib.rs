use crossterm::event::{self, poll, read, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, is_raw_mode_enabled};
use log::debug;
use scopeguard::defer;
use std::env;
use std::io::IsTerminal;
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};
use thiserror::Error;
#[cfg(target_os = "windows")]
use {
    std::sync::OnceLock,
    winapi::um::consoleapi::SetConsoleMode,
    winapi::um::handleapi::INVALID_HANDLE_VALUE,
    winapi::um::processenv::GetStdHandle,
    winapi::um::winbase::STD_OUTPUT_HANDLE,
    winapi::um::wincon::{self, ENABLE_VIRTUAL_TERMINAL_PROCESSING},
};

/// Terminal
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Terminal {
    Screen,
    Tmux,
    XtermCompatible,
    Windows,
    Emacs,
}

/// 16bit RGB color
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u16,
    pub g: u16,
    pub b: u16,
}

/// Background theme
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

/// Error
#[derive(Error, Debug)]
pub enum Error {
    #[error("io error")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("parse error")]
    Parse(String),
    #[error("unsupported")]
    Unsupported,
}

/// get detected terminal
#[cfg(not(target_os = "windows"))]
pub fn terminal() -> Terminal {
    if env::var("INSIDE_EMACS").is_ok() {
        return Terminal::Emacs;
    }

    if env::var("TMUX").is_ok() || env::var("TERM").is_ok_and(|x| x.starts_with("tmux-")) {
        Terminal::Tmux
    } else {
        let is_screen = if let Ok(term) = env::var("TERM") {
            term.starts_with("screen")
        } else {
            false
        };
        if is_screen {
            Terminal::Screen
        } else {
            Terminal::XtermCompatible
        }
    }
}

/// get detected terminal
#[cfg(target_os = "windows")]
pub fn terminal() -> Terminal {
    // Although xterm OSC is MS's roadmap, as of 2024-10-16, only Windows Terminal 1.22 (preview)
    // supports *querying* rgb values. In the mean time, there is effectively no way to query
    // Windows color schemes.
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        if term_program == "vscode" {
            return Terminal::XtermCompatible;
        }
    }

    if env::var("INSIDE_EMACS").is_ok() {
        return Terminal::Emacs;
    }

    // Windows Terminal is Xterm-compatible
    // https://github.com/microsoft/terminal/issues/3718.
    // But this excludes OSC 10/11 colour queries until Windows Terminal 1.22
    // https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release/:
    // "Applications can now query ... the default foreground (OSC 10 ?) [and] background (OSC 11 ?)"
    // Don't use WT_SESSION for this purpose:
    // https://github.com/Textualize/rich/issues/140
    // if env::var("WT_SESSION").is_ok() {
    if enable_virtual_terminal_processing() {
        debug!(
            r#"This Windows terminal supports virtual terminal processing
(but not OSC 10/11 colour queries if prior to Windows Terminal 1.22 Preview of August 2024)\r"#
        );
        Terminal::XtermCompatible
    } else {
        debug!("Terminal::Windows\r");
        Terminal::Windows
    }
}

/// get background color by `RGB`
#[cfg(not(target_os = "windows"))]
pub fn rgb(timeout: Duration) -> Result<Rgb, Error> {
    let term = terminal();
    let rgb = match term {
        Terminal::Emacs => Err(Error::Unsupported),
        _ => from_xterm(term, timeout),
    };
    let fallback = from_env_colorfgbg();
    if rgb.is_ok() {
        rgb
    } else if fallback.is_ok() {
        fallback
    } else {
        rgb
    }
}

/// get background color by `RGB`
#[cfg(target_os = "windows")]
pub fn rgb(timeout: Duration) -> Result<Rgb, Error> {
    let term = terminal();
    let rgb = match term {
        Terminal::Emacs => Err(Error::Unsupported),
        Terminal::XtermCompatible => from_xterm(term, timeout),
        _ => from_winapi(),
    };
    let fallback = from_env_colorfgbg();
    debug!("rgb={rgb:?}, fallback={fallback:?}\r");
    if rgb.is_ok() {
        rgb
    } else if fallback.is_ok() {
        fallback
    } else {
        rgb
    }
}

/// get terminal latency
#[cfg(not(target_os = "windows"))]
pub fn latency(timeout: Duration) -> Result<Duration, Error> {
    let term = terminal();
    match term {
        Terminal::Emacs => Ok(Duration::from_millis(0)),
        _ => xterm_latency(timeout),
    }
}

/// get terminal latency
#[cfg(target_os = "windows")]
pub fn latency(timeout: Duration) -> Result<Duration, Error> {
    let term = terminal();
    match term {
        Terminal::Emacs => Ok(Duration::from_millis(0)),
        Terminal::XtermCompatible => xterm_latency(timeout),
        _ => Ok(Duration::from_millis(0)),
    }
}

/// get background color by `Theme`
pub fn theme(timeout: Duration) -> Result<Theme, Error> {
    let rgb = rgb(timeout)?;

    // ITU-R BT.601
    let y = rgb.r as f64 * 0.299 + rgb.g as f64 * 0.587 + rgb.b as f64 * 0.114;

    if y > 32768.0 {
        Ok(Theme::Light)
    } else {
        Ok(Theme::Dark)
    }
}

// Function to enable virtual terminal processing for Windows
#[cfg(target_os = "windows")]
fn enable_virtual_terminal_processing() -> bool {
    static ENABLE_VT_PROCESSING: OnceLock<bool> = OnceLock::new();
    *ENABLE_VT_PROCESSING.get_or_init(|| unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle != INVALID_HANDLE_VALUE {
            let mut mode: u32 = 0;
            if winapi::um::consoleapi::GetConsoleMode(handle, &mut mode) != 0 {
                // Try to set virtual terminal processing mode
                if SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0 {
                    // Success in enabling VT
                    debug!("Successfully enabled Virtual Terminal Processing.\r");
                    return true;
                } else {
                    // Failed to enable VT, optionally log error
                    debug!("Failed to enable Virtual Terminal Processing.\r");
                }
            }
        }
        // Return false if enabling VT failed
        false
    })
}

fn from_xterm(term: Terminal, timeout: Duration) -> Result<Rgb, Error> {
    if !std::io::stdin().is_terminal()
        || !std::io::stdout().is_terminal()
        || !std::io::stderr().is_terminal()
    {
        // Not a terminal, so don't try to read the current background color.
        return Err(Error::Unsupported);
    }

    let raw_before = is_raw_mode_enabled()?;

    defer! {
        let is_raw = match is_raw_mode_enabled() {
            Ok(val) => val,
            Err(e) => {
                debug!("Failed to check raw mode status: {:?}\r", e);
                return;
            }
        };

        if is_raw == raw_before {
            debug!("Raw mode status unchanged from raw={raw_before}.\r");
        } else if let Err(e) = restore_raw_status(raw_before) {
            debug!("Failed to restore raw mode: {e:?} to raw={raw_before}\r");
        } else {
            debug!("Raw mode restored to previous state (raw={raw_before}).\r");
        }

        if let Err(e) = clear_stdin() {
            debug!("Failed to clear stdin: {e:?}\r");
        } else {
            debug!("Cleared any excess from stdin.\r");
        }
    }

    if !raw_before {
        terminal::enable_raw_mode()?;
    }

    let mut stderr = io::stderr();

    #[cfg(target_os = "windows")]
    {
        if !enable_virtual_terminal_processing() {
            debug!(
                "Virtual Terminal Processing could not be enabled. Falling back to default behavior.\r"
            );
            return from_winapi();
        }
    }

    // Query by XTerm control sequence
    let query = match term {
        Terminal::Tmux => "\x1bPtmux;\x1b\x1b]11;?\x07\x1b\\",
        Terminal::Screen => "\x1bP\x1b]11;?\x07\x1b\\",
        _ => "\x1b]11;?\x1b\\",
    };

    // Send query
    write!(stderr, "{query}")?;
    stderr.flush()?;

    let mut response = String::new();
    let start_time = Instant::now();

    // Main loop for capturing terminal response
    loop {
        if start_time.elapsed() > timeout {
            debug!("After timeout, found response={response}\r");
            if response.contains("rgb:") {
                let rgb_slice = decode_unterminated(&response)?;
                debug!("Found a valid response {rgb_slice} in pre-timeout check despite unrecognized terminator in response code {response:#?}\r");
                return parse_response(rgb_slice, start_time);
            }
            debug!("Failed to capture response\r");
            return Err(io::Error::new(io::ErrorKind::TimedOut, "timeout").into());
        }

        // Replaced expensive async_std with blocking loop. Terminal normally responds
        // fast or not at all, and in the latter case we still have the timeout on the
        // main loop.
        if poll(Duration::from_millis(100))? {
            // Read the next event.
            // Replaced stdin read that was consuming legit user input in Windows
            // with non-blocking crossterm read event.
            if let Event::Key(key_event) = event::read()? {
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Char('\\'), KeyModifiers::ALT | KeyModifiers::NONE)   // ST
                    | (KeyCode::Char('g'), KeyModifiers::CONTROL)   // BEL
                    // Insurance in case BEL is not recognosed as ^g
                    | (KeyCode::Char('\u{0007}'), KeyModifiers::NONE)   //BEL
                    => {
                        debug!("End of response detected ({key_event:?}).\r");
                        // response.push('\\');
                        // debug!("response={response}\r");
                        return parse_response(&response, start_time);
                    }
                    // Append other characters to buffer
                    (KeyCode::Char(c), KeyModifiers::NONE) => {
                        debug!("pushing {c}\r");
                        response.push(c);
                    }
                    _ => {
                        // Ignore other keys
                        debug!("ignoring {key_event:?}\r");
                    }
                }
            }
        }
    }
}

fn decode_unterminated(response: &str) -> Result<&str, Error> {
    let resp_start = response.find("rgb:").ok_or(Error::Parse(
        "Required string `rgb:` not found in response".to_string(),
    ))?;
    let mid = resp_start + 4;
    // Point after "rgb:"
    let raw_rgb_slice = response.split_at(mid).1;
    // slash-delimited r/g/b string with any trailing characters
    debug!("raw_rgb_slice={raw_rgb_slice}\r");

    // Identify where to trim trailing characters, by assuming the slash-delimited colour specifiers
    // are all supposed to be the same length. I.e. trim after 3 specifiers and 2 delimiters.
    let fragments = raw_rgb_slice.splitn(3, '/').collect::<Vec<_>>();

    if fragments.len() < 3 {
        // debug!("Incomplete response `{response}`: does not contain two forward slashes\r");
        return Err(Error::Parse(format!(
            "Incomplete response `{response}`: does not contain two forward slashes"
        )));
    }
    let frag_len = fragments[0].len();
    if fragments[1].len() != frag_len || fragments[2].len() < frag_len {
        // debug!("Can't safely reconstitute unterminated response `{response}`from fragments of unequal length\r");
        return Err(Error::Parse(format!("Can't safely reconstitute unterminated response `{response}`from fragments of unequal length")));
    }

    // "Trim" extraneous trailing characters by excluding them from slice
    let rgb_str_len = frag_len * 3 + 2;
    let rgb_slice = &response[resp_start..mid + rgb_str_len];
    Ok(rgb_slice)
}

fn parse_response(response: &str, start_time: Instant) -> Result<Rgb, Error> {
    // debug!("response={response}\r");
    let (r, g, b) = extract_rgb(response)?;
    let elapsed = start_time.elapsed();
    debug!("Elapsed time: {:.2?}\r", elapsed);
    // debug!("Rgb {{ r, g, b }} = {:?}\r", Rgb { r, g, b });
    Ok(Rgb { r, g, b })
}

fn extract_rgb(response: &str) -> Result<(u16, u16, u16), Error> {
    let rgb_str = response
        .split_at(
            response.find("rgb:").ok_or(Error::Parse(
                "Could not find 'rgb:' in terminal response string".to_string(),
            ))? + 4,
        )
        .1;
    let (r, g, b) = decode_x11_color(rgb_str)?;
    // debug!("(r, g, b)=({r}, {g}, {b})\r");
    Ok((r, g, b))
}

fn restore_raw_status(raw_before: bool) -> Result<(), Error> {
    let raw_now = is_raw_mode_enabled()?;
    if raw_now == raw_before {
        return Ok(());
    }
    if raw_before {
        terminal::enable_raw_mode()?;
    } else {
        terminal::disable_raw_mode()?;
    }
    Ok(())
}

/// Discard any unread input returned by the OSC 11 query.
///
/// # Errors
///
/// This function will return an error if Rust has decided that the "terminal" is not a terminal.
// Helper function to discard extra characters
fn clear_stdin() -> Result<(), Box<dyn std::error::Error>> {
    while poll(Duration::from_millis(10))? {
        if let Event::Key(c) = read()? {
            // Discard the input by simply reading it
            debug!("discarding char{c:x?}\r");
        }
    }
    Ok(())
}

/// Seems to be for Rxvt terminal emulator only.
fn from_env_colorfgbg() -> Result<Rgb, Error> {
    let var = env::var("COLORFGBG").map_err(|_| Error::Unsupported)?;
    let fgbg: Vec<_> = var.split(";").collect();
    let bg = fgbg.get(1).ok_or(Error::Unsupported)?;
    let bg = u8::from_str_radix(bg, 10).map_err(|_| Error::Parse(String::from(var)))?;

    // rxvt default color table
    #[allow(clippy::match_same_arms)]
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

    Ok(Rgb {
        r: r * 256,
        g: g * 256,
        b: b * 256,
    })
}

fn xterm_latency(timeout: Duration) -> Result<Duration, Error> {
    let query = "\x1b[5n";
    let mut stderr = io::stderr();

    let raw_before = is_raw_mode_enabled()?;

    defer! {
        let is_raw = match is_raw_mode_enabled() {
            Ok(val) => val,
            Err(e) => {
                debug!("Failed to check raw mode status: {:?}\r", e);
                return;
            }
        };

        if is_raw == raw_before {
            debug!("Raw mode status unchanged from raw={raw_before}.\r");
        } else if let Err(e) = restore_raw_status(raw_before) {
            debug!("Failed to restore raw mode: {e:?} to raw={raw_before}\r");
        } else {
            debug!("Raw mode restored to previous state (raw={raw_before}).\r");
        }

        if let Err(e) = clear_stdin() {
            debug!("Failed to clear stdin: {e:?}\r");
        } else {
            debug!("Cleared any excess from stdin.\r");
        }
    }

    if !raw_before {
        terminal::enable_raw_mode()?;
    }

    // Send the query
    stderr.write_all(query.as_bytes())?;
    stderr.flush()?;

    let start_time = Instant::now();

    // Enter raw mode to capture input
    terminal::enable_raw_mode()?;
    let mut stdin = io::stdin();
    let mut response = String::new();

    // Main loop to capture response
    loop {
        // Check for timeout
        if start_time.elapsed() > timeout {
            terminal::disable_raw_mode()?; // Clean up raw mode
            return Err(io::Error::new(io::ErrorKind::TimedOut, "timeout").into());
        }

        // Non-blocking read attempt from stdin
        let mut buffer = [0u8; 1];
        if stdin.read(&mut buffer).is_ok() {
            response.push(buffer[0] as char);

            // End the loop once we detect the 'n' character
            if response.ends_with('n') {
                let elapsed = start_time.elapsed();
                debug!("Latency full response: [{response:x?}]\r");
                // debug!("Elapsed time: {elapsed:?}\r");

                return Ok(elapsed);
            }
        }
    }
}

fn decode_x11_color(s: &str) -> Result<(u16, u16, u16), Error> {
    fn decode_hex(s: &str) -> Result<u16, Error> {
        let len = s.len() as u32;
        let mut ret = u16::from_str_radix(s, 16).map_err(|_| Error::Parse(String::from(s)))?;
        ret = ret << ((4 - len) * 4);
        Ok(ret)
    }

    let rgb: Vec<_> = s.split('/').collect();

    let r = rgb.get(0).ok_or_else(|| Error::Parse(String::from(s)))?;
    let g = rgb.get(1).ok_or_else(|| Error::Parse(String::from(s)))?;
    let b = rgb.get(2).ok_or_else(|| Error::Parse(String::from(s)))?;
    let r = decode_hex(r)?;
    let g = decode_hex(g)?;
    let b = decode_hex(b)?;

    Ok((r, g, b))
}

/// Try to determine the background colour from the legacy Windows Console interface.
/// Unfortunately, unless the colour was explicitly set by that interface, it will
/// just return the default of rgb(0,0,0). This renders it effectively useless for
/// modern Windows.
///
/// # Errors
///
/// This function will bubble up any errors returned by the Windows API.
#[cfg(target_os = "windows")]
fn from_winapi() -> Result<Rgb, Error> {
    debug!("In from_winapi()\r");
    let info = unsafe {
        let handle = winapi::um::processenv::GetStdHandle(winapi::um::winbase::STD_OUTPUT_HANDLE);
        let mut info: wincon::CONSOLE_SCREEN_BUFFER_INFO = Default::default();
        wincon::GetConsoleScreenBufferInfo(handle, &mut info);
        info
    };

    debug!("info.wAttributes={:x?}\r", info.wAttributes);

    let r = (wincon::BACKGROUND_RED & info.wAttributes) != 0;
    let g = (wincon::BACKGROUND_GREEN & info.wAttributes) != 0;
    let b = (wincon::BACKGROUND_BLUE & info.wAttributes) != 0;
    let i = (wincon::BACKGROUND_INTENSITY & info.wAttributes) != 0;

    let r: u8 = r as u8;
    let g: u8 = g as u8;
    let b: u8 = b as u8;
    let i: u8 = i as u8;

    let (r, g, b) = match (r, g, b, i) {
        (0, 0, 0, 0) => (0, 0, 0),
        (1, 0, 0, 0) => (128, 0, 0),
        (0, 1, 0, 0) => (0, 128, 0),
        (1, 1, 0, 0) => (128, 128, 0),
        (0, 0, 1, 0) => (0, 0, 128),
        (1, 0, 1, 0) => (128, 0, 128),
        (0, 1, 1, 0) => (0, 128, 128),
        (1, 1, 1, 0) => (192, 192, 192),
        (0, 0, 0, 1) => (128, 128, 128),
        (1, 0, 0, 1) => (255, 0, 0),
        (0, 1, 0, 1) => (0, 255, 0),
        (1, 1, 0, 1) => (255, 255, 0),
        (0, 0, 1, 1) => (0, 0, 255),
        (1, 0, 1, 1) => (255, 0, 255),
        (0, 1, 1, 1) => (0, 255, 255),
        (1, 1, 1, 1) => (255, 255, 255),
        _ => unreachable!(),
    };

    Ok(Rgb {
        r: r * 256,
        g: g * 256,
        b: b * 256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_x11_color() {
        let s = "0000/0000/0000";
        assert_eq!((0, 0, 0), decode_x11_color(s).unwrap());

        let s = "1111/2222/3333";
        assert_eq!((0x1111, 0x2222, 0x3333), decode_x11_color(s).unwrap());

        let s = "111/222/333";
        assert_eq!((0x1110, 0x2220, 0x3330), decode_x11_color(s).unwrap());

        let s = "11/22/33";
        assert_eq!((0x1100, 0x2200, 0x3300), decode_x11_color(s).unwrap());

        let s = "1/2/3";
        assert_eq!((0x1000, 0x2000, 0x3000), decode_x11_color(s).unwrap());
    }
}
