fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .out_dir("src/generated") // you can change the generated code's location
        .compile(
            &[
                // "./fiber-proto/types.proto", uncomment this line if you have a dependency on types.proto
                // "./fiber-proto/eth.proto", uncomment this line if you have a dependency on eth.proto
                "./fiber-proto/api.proto",
            ],
            &["./fiber-proto"], // specify the root location to search proto dependencies
        )
        .unwrap();
    Ok(())
}
