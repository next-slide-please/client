extern crate vergen;

use vergen::{generate_cargo_keys, ConstantsFlags};

fn main() {
    // Setup the flags, toggling off the 'SEMVER_FROM_CARGO_PKG' flag
    // Generate the 'cargo:' key output
    generate_cargo_keys(ConstantsFlags::all())
        .expect("Unable to generate the cargo keys!");
}
