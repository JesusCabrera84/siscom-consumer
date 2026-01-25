use figlet_rs::FIGfont;

pub fn print_banner() {
    println!("##########################################################################################");
    match FIGfont::from_file("assets/3d.flf") {
        Ok(standard_font) => {
            let figure = standard_font.convert("SISCOM     BOOT");
            if let Some(fig) = figure {
                println!("{}", fig);
            }
        }
        Err(e) => {
            println!("Warning: Could not load font file: {}", e);
        }
    }
    println!("##########################################################################################");
}
