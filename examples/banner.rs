//! Prints the banner in 24-bit color, for previews: `cargo run --example banner`.

fn main() {
    print!("{}", visor::banner::render(visor::banner::Palette::TrueColor));
}
