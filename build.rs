use async_trait::async_trait;
use failure::{format_err, Error};
use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;

async fn print_all<T>(stream: Option<T>) -> Result<(), Error>
where
    T: AsyncRead + Unpin,
{
    if let Some(stream) = stream {
        let mut lines = BufReader::new(stream).lines();
        while let Some(line) = lines.next_line().await? {
            eprintln!("{}", line);
        }
    }
    Ok(())
}

#[async_trait]
trait RunIt {
    async fn run_it(&mut self, err: &str) -> Result<(), Error>;
}

#[async_trait]
impl RunIt for Command {
    async fn run_it(&mut self, err: &str) -> Result<(), Error> {
        self.env("RUST_LOG", "0");
        self.stdin(Stdio::null());
        self.stdout(Stdio::piped());
        self.stderr(Stdio::piped());
        let mut child = self.spawn()?;
        let (_, _, res) = tokio::join!(
            print_all(child.stdout.take()),
            print_all(child.stderr.take()),
            child,
        );
        if !res?.success() {
            Err(format_err!("{}", err))
        } else {
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    Command::new("wasm-pack")
        .args(&["build", "--target", "web"])
        .current_dir("ui")
        .run_it("Can't compile UI")
        .await?;

    Command::new("rollup")
        .args(&["./main.js", "--format", "iife", "--file", "./pkg/bundle.js"])
        .current_dir("ui")
        .run_it("Can't rollup UI")
        .await?;

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
            "styles.css",
            "pkg/bundle.js",
            "pkg/tody_chat_ui_bg.wasm",
        ])
        .current_dir("ui")
        .run_it("Can't pack UI")
        .await?;

    if cfg!(feature = "refresh") {
        Command::new("touch")
            .args(&["build.rs"])
            .run_it("Can't touch the build file")
            .await?;
    }
    Ok(())
}
