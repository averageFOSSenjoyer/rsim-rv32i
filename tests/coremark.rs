use rsim_rv32i::backend::core::Core;
use std::fs::File;

#[test]
fn coremark() {
    let commit_file = File::create("./tests/coremark.log").unwrap();
    let core = Core::new(4, Some(commit_file));
    let coremark = std::fs::read("./tests/coremark.elf").unwrap();
    core.load_elf(coremark.as_slice());

    core.run_end(Some(|| {}));
}
