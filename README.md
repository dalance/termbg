# termbg
A Rust library for terminal background color detection.
The detected color is provided by RGB or theme ( dark or light ).

[![Actions Status](https://github.com/dalance/termbg/workflows/Rust/badge.svg)](https://github.com/dalance/termbg/actions)
[![Crates.io](https://img.shields.io/crates/v/termbg.svg)](https://crates.io/crates/termbg)
[![Docs.rs](https://docs.rs/termbg/badge.svg)](https://docs.rs/termbg)

## Verified terminals

* Alacritty
* GNOME Terminal
* GNU Screen
* macOS terminal
* MATE Terminal
* mintty
* RLogin
* rxvt-unicode
* sakura
* Tera Term
* Terminator
* tmux
* xfce4-terminal
* xterm
* Win32 console

"Windows Terminal" will be supported in a future release: https://github.com/microsoft/terminal/issues/3718.

If you check other terminals, please report through [issue](https://github.com/dalance/termbg/issues).

## Unsupported terminals

* LilyTerm
* Poderosa
* PuTTY
* QTerminal

## Usage

```Cargo.toml
[dependencies]
termbg = "0.2.0"
```

## Example

```rust
fn main() {
    let timeout = std::time::Duration::from_millis(100);

    println!("Check terminal background color");
    let term = termbg::terminal();
    let rgb = termbg::rgb(timeout);
    let theme = termbg::theme(timeout);

    println!("  Term : {:?}", term);

    match rgb {
        Ok(rgb) => {
            println!("  Color: R={:x}, G={:x}, B={:x}", rgb.r, rgb.g, rgb.b);
        }
        Err(e) => {
            println!("  Color: detection failed {:?}", e);
        }
    }

    match theme {
        Ok(theme) => {
            println!("  Theme: {:?}", theme);
        }
        Err(e) => {
            println!("  Theme: detection failed {:?}", e);
        }
    }
}
```

## Check program

This crate provides a simple program to check.

```console
$ cargo run
Check terminal background color
  Term : Tmux
  Color: R=0, G=0, B=0
  Theme: Dark
```

## Detecting mechanism

If the terminal is win32 console, WIN32API is used for detection.
If the terminal is xterm compatible, "Xterm Control Sequences" is used.

The detected RGB is converted to YCbCr.
If Y > 0.5, the theme is detected as "light", otherwise "dark".
