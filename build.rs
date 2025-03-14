use vergen_gix::{BuildBuilder, Emitter, GixBuilder, SysinfoBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build = BuildBuilder::all_build()?;
    let gitcl = GixBuilder::default().all().sha(true).build()?;
    let si = SysinfoBuilder::all_sysinfo()?;

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&gitcl)?
        .add_instructions(&si)?
        .emit()?;

    Ok(())
}
