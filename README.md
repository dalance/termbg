# termbg
A Rust library for terminal background color detection.
The detected color is provided by RGB or theme ( dark or light ).

[![Actions Status](https://github.com/dalance/termbg/workflows/Rust/badge.svg)](https://github.com/dalance/termbg/actions)
[![Crates.io](https://img.shields.io/crates/v/termbg.svg)](https://crates.io/crates/termbg)
[![Docs.rs](https://docs.rs/termbg/badge.svg)](https://docs.rs/termbg)

## Verified terminal

* mintty 3.0.6
* rxvt-unicode 9.21
* GNOME Terminal 3.28.2
* xterm patch 295
* tmux 3.1b
* screen 4.01.00

If you check other terminals, please report through [issue](https://github.com/dalance/termbg/issues).

## Usage

```Cargo.toml
[dependencies]
termbg = "0.1.0"
```

## Example

```rust
fn main() {
    println!("Check terminal background color");
    let rgb = termbg::rgb();
    let theme = termbg::theme();

    if let Some(rgb) = rgb {
        println!("  Color: R={:x}, G={:x}, B={:x}", rgb.r, rgb.g, rgb.b);
    } else {
        println!("  Color: detection failed");
    }

    if let Some(theme) = theme {
        println!("  Theme: {:?}", theme);
    } else {
        println!("  Theme: detection failed");
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

If the terminal is rxvt compatible ( `COLORFGBG` environment variable is defined ),
background color is detected from `COLORFGBG`.

On the other hand, if the terminal is xterm compatible, "Xterm Control Sequences" is used for detection.

The detected RGB is converted to YCbCr.
If Y > 0.5, the theme is detected as "light", otherwise "dark".
