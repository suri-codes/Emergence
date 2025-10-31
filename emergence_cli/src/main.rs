use std::{fs, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::Result;
use emergence_zk::{Kasten, Tag, Zettel, ZettelBuilder};

use crate::args::{CliArgs, Commands};

mod args;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = CliArgs::parse();

    match args.command {
        Commands::Init(args) => {
            fs::create_dir(format!("./{}", args.name))?;
            fs::create_dir(format!("./{}/.emergence/", args.name))?;
        }

        Commands::New => {}
    }

    // let test_root = PathBuf::from("./test");

    // let zk_b = ZettelBuilder::new(test_root.clone());

    // let zk = zk_b
    //     .name("kill adrien!")
    //     .add_tag(Tag::new("test", "color!").expect("color"))
    //     .add_tag(Tag::new("death", "color!").expect("color"))
    //     .add_tag(Tag::new("mediatok", "color!").expect("color"))
    //     .content("Adrian is just so butt, [PENIS!](./cSRPIvjBQJfv6gjSthYD5.md)")
    //     .build()
    //     .expect("lol");

    // let _zettel: Zettel = zk.path.as_path().try_into().expect("lol");

    // let kasten = Kasten::generate(test_root).expect("Whateva");

    // println!("Kasten: {kasten:#?}");
    Ok(())
}
