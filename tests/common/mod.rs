use std::{
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
static COUNTER: AtomicU64 = AtomicU64::new(0);
pub struct TempDir(pub PathBuf);
impl TempDir {
    pub fn new() -> Self {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "adc-enob-{}-{time}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
pub fn run(args: &[&str], dir: &TempDir) -> Output {
    Command::new(env!("CARGO_BIN_EXE_adc-oversampling-enob"))
        .args(args)
        .arg("--out")
        .arg(&dir.0)
        .output()
        .unwrap()
}
