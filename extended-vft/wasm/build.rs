use extended_vft_app::Program;
use std::{env, path::PathBuf};

fn main() {
    gwasm_builder::build();

    let idl_file_path =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("extended_vft.idl");

    // Generate IDL file for the app
    sails_idl_gen::generate_idl_to_file::<Program>(&idl_file_path).unwrap();

    // Generate client code from IDL file
    sails_client_gen::generate_client_from_idl(
        &idl_file_path,
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("extended_vft_client.rs"),
        None,
    )
    .unwrap();
}
