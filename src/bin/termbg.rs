fn main() {
    println!("Check terminal background color");
    let term = termbg::terminal();
    let rgb = termbg::rgb();
    let theme = termbg::theme();

    println!("  Term : {:?}", term);

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
