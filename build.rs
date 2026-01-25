use std::io::Result;

fn main() -> Result<()> {
    // Compilar el archivo protobuf y generar en src/
    prost_build::Config::new()
        .out_dir("src/")
        .compile_protos(&["siscom.proto"], &["."])?;
    Ok(())
}