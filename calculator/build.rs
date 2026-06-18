use std::error::Error;
use std::{env, path::PathBuf};

/**
 * The `main` function is the entry point of the build script. 
 * It is responsible for compiling the protobuf files and generating the necessary Rust code for the 
 * gRPC services defined in the protobuf files.
 */
fn main() -> Result<(), Box<dyn Error>> {
    /*
        The `OUT_DIR` environment variable is used to specify the output directory for the generated code. 
        The `PathBuf::from` function is used to create a new `PathBuf` instance from the `OUT_DIR` 
        environment variable.
     */
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    /*
        The `tonic_build::configure` function is used to configure the protobuf compilation settings.
        The `file_descriptor_set_path` method is used to specify the path for the generated file descriptor set.
        The `compile` method is used to compile the protobuf files.
     */
    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("calculator_descriptor.bin"))
        .compile(&["proto/calculator.proto"], &["proto"])?;
        
    /*
        The `tonic_build::compile_protos` function is called to compile the protobuf 
        file located at "proto/calculator.proto".
     */
    tonic_build::compile_protos("proto/calculator.proto")?;

    Ok(())
}