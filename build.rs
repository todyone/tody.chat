use failure::{format_err, Error};
use std::env;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};

trait RunIt {
    fn run_it(&mut self, err: &str) -> Result<(), Error>;
}

impl RunIt for Command {
    fn run_it(&mut self, err: &str) -> Result<(), Error> {
        self.stdin(Stdio::null());
        self.stdout(Stdio::piped());
        self.stderr(Stdio::piped());
        let mut child = self.spawn()?;
        // Important! We have to read stderr first, because wasm-pack required that.
        if let Some(mut err) = child.stderr.take() {
            let mut out = String::new();
            err.read_to_string(&mut out)?;
            eprintln!("{}", out);
        }
        if let Some(out) = child.stdout.take() {
            let buf = BufReader::new(out);
            for line in buf.lines() {
                eprint!("{}", line?);
            }
        }
        if !child.wait()?.success() {
            Err(format_err!("{}", err))
        } else {
            Ok(())
        }
    }
}

fn main() -> Result<(), Error> {
    Command::new("wasm-pack")
        .args(&["build", "--target", "web"])
        .current_dir("ui")
        .run_it("Can't compile UI")?;

    Command::new("rollup")
        .args(&["./main.js", "--format", "iife", "--file", "./pkg/bundle.js"])
        .current_dir("ui")
        .run_it("Can't rollup UI")?;

    let out_path = PathBuf::from(env::var("OUT_DIR")?);
    let tar_path = out_path.join("ui.tar.gz");
    let tar_path = tar_path
        .to_str()
        .ok_or_else(|| format_err!("can't create path to archive"))?;

    Command::new("tar")
        .args(&[
            "-cvzf",
            tar_path,
            "index.html",
            "pkg/bundle.js",
            "pkg/tody_chat_ui_bg.wasm",
        ])
        .current_dir("ui")
        .run_it("Can't pack UI")?;

    if cfg!(feature = "refresh") {
        Command::new("touch")
            .args(&["build.rs"])
            .run_it("Can't touch the build file")?;
    }
    Ok(())
}
