//! Builds the wallpaper changer by downloading the Montserrat font.
use std::env;
use std::fs::File;
use std::io::copy;
use std::io::Write;
use std::path::Path;

#[cfg(clippy)]
fn main() {}

#[cfg(not(clippy))]
fn main() {
    // Directory where the font will be downloaded
    let out_dir_env = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_env);

    // URL of the Montserrat font
    let font_url = "https://raw.githubusercontent.com/JulietaUla/Montserrat/refs/heads/master/fonts/ttf/Montserrat-Bold.ttf";

    // Download the font
    let response = ureq::get(font_url)
        .call()
        .expect("Failed to download Montserrat font");

    assert!(
        response.status() == 200,
        "Failed to download Montserrat font: HTTP {}",
        response.status()
    );

    // Write the font to a file
    let font_path = out_dir.join("Montserrat-Bold.ttf");
    let mut font_file = File::create(&font_path).expect("Failed to create font file");
    copy(&mut response.into_body().into_reader(), &mut font_file)
        .expect("Failed to write font file");

    // Output the path to the downloaded font so it can be used in the main program
    let mut file = File::create(Path::new(&out_dir).join("font_path.txt")).unwrap();
    writeln!(file, "{}", font_path.to_str().unwrap()).unwrap();
}
