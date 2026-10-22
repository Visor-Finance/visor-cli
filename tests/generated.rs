//! Keeps the shell completions and the man page in `dist-assets/` in step
//! with the commands. dist packs them into every release archive.
//!
//! After changing a command or flag, regenerate them with
//! `VISOR_UPDATE_GENERATED=1 cargo test --test generated`.

use std::path::PathBuf;

use clap::CommandFactory;
use clap_complete::Shell;
use visor::cli::Cli;

fn generated() -> Vec<(PathBuf, Vec<u8>)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dist-assets");
    let mut files = Vec::new();
    for (shell, name) in [
        (Shell::Bash, "visor.bash"),
        (Shell::Zsh, "_visor"),
        (Shell::Fish, "visor.fish"),
    ] {
        let mut out = Vec::new();
        clap_complete::generate(shell, &mut Cli::command(), "visor", &mut out);
        files.push((dir.join("completions").join(name), out));
    }
    let mut man = Vec::new();
    clap_mangen::Man::new(Cli::command())
        .render(&mut man)
        .expect("man page renders");
    files.push((dir.join("man").join("visor.1"), man));
    files
}

#[test]
fn completions_and_man_page_are_current() {
    let update = std::env::var_os("VISOR_UPDATE_GENERATED").is_some();
    let mut stale = Vec::new();
    for (path, content) in generated() {
        if update {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &content).unwrap();
        } else if std::fs::read(&path).ok().as_deref() != Some(content.as_slice()) {
            stale.push(path.display().to_string());
        }
    }
    assert!(
        stale.is_empty(),
        "out of date: {stale:?}\nRun: VISOR_UPDATE_GENERATED=1 cargo test --test generated"
    );
}
