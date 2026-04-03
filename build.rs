fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate the gRPC code from the proto definition file
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(&["proto/orders.proto"], &["proto"])?;

    println!("cargo:rerun-if-changed=proto/orders.proto");
    println!("cargo:rerun-if-changed=proto");
    Ok(())
}
