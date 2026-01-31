use std::io::Result;

fn main() -> Result<()> {
    // Set CMAKE_ARGS for librdkafka SASL support
    println!("cargo:rustc-env=CMAKE_ARGS=-DWITH_SASL=ON -DWITH_SSL=ON");

    // Compilar el archivo protobuf y generar en src/
    prost_build::Config::new()
        .out_dir("src/")
        .compile_protos(&["siscom.proto"], &["."])?;
    Ok(())
}