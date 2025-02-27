#![feature(exit_status_error)]


use std::process::Command;


fn main() {

    let lighthousemc_editor_commit    = String::from_utf8(Command::new("git").args(["rev-parse", "HEAD"]).output().unwrap().stdout).unwrap().replace("\n", "");
    let lighthousemc_editor_timestamp = String::from_utf8(Command::new("git").args(["show", "-s", "--format=%ci", "HEAD"]).output().unwrap().stdout).unwrap().replace("\n", "");
    println!("cargo:rustc-env=LIGHTHOUSEMC_EDITOR_COMMIT={} {}", &lighthousemc_editor_commit[..9], lighthousemc_editor_timestamp.split(" ").nth(0).unwrap());
    println!("cargo:rustc-env=LIGHTHOUSEMC_EDITOR_COMMIT_HASH={}", lighthousemc_editor_commit);

    Command::new("wasm-pack").args(["build", "--target", "web"]).current_dir("lighthousemc-editor-frontend").status().unwrap().exit_ok().unwrap();
    println!("cargo::rerun-if-changed=lighthousemc-editor-common");
    println!("cargo::rerun-if-changed=lighthousemc-editor-frontend");

}
