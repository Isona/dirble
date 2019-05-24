use vergen::{
    ConstantsFlags,
    generate_cargo_keys,
};

fn main() {
    let mut flags = ConstantsFlags::all();
    flags.toggle(ConstantsFlags::SEMVER_FROM_CARGO_PKG);

    generate_cargo_keys(ConstantsFlags::all())
        .expect("Unable to generate Cargo env keys");
}
