use std::{
    env::{self, current_dir},
    process::Command,
};

use clap::Parser as _;
use color_eyre::{eyre::Result, owo_colors::OwoColorize as _};
use emergence_zk::{Kasten, Tag, Zettel, ZettelBuilder};

use crate::args::{CliArgs, Commands};

mod args;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = CliArgs::parse();

    match args.command {
        Commands::Init(args) => {
            Kasten::new(&args.name)?;

            let zk_created = "ZettelKasten Created!".green();

            let get_started_preface = "To get started, try running";

            let tmp = format!("cd {}\nezk new", args.name);
            let tmp_str = tmp.as_str();
            let get_started_cmd = tmp_str.green();

            println!("{zk_created}\n{get_started_preface}\n\n{get_started_cmd}");

            Ok(())
        }

        Commands::New(args) => {
            let pwd = current_dir()?;

            // make sure this directory is a kasten, might be a better way to do this
            let _: Kasten = Kasten::parse(&pwd)
                .inspect_err(|e| eprintln!("You arent in a valid kasten! {e}"))?;

            let mut zb = ZettelBuilder::new(pwd);

            if let Some(name) = args.name {
                zb.name(name);
            }

            for tag in args.tags {
                zb.add_tag(Tag::new(tag, "penis")?);
            }

            let z: Zettel = zb.build()?;

            let editor = env::var("EDITOR")
                .or_else(|_| env::var("VISUAL"))
                .unwrap_or_else(|_| "vim".to_owned());

            Command::new(editor).arg(&z.path).status()?;

            Ok(())
        }
    }
}
