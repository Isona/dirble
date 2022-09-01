use vergen::{Config, ShaKind, TimestampKind};

fn main() {
    let mut config = Config::default();
    *config.git_mut().sha_kind_mut() = ShaKind::Short;
    *config.build_mut().kind_mut() = TimestampKind::DateOnly;

    vergen::vergen(config).expect("Couldn't generate keys");
}
