use emergence_zk::{
    ZettelId,
    entities::{tag, zettel, zettel_tag},
    entity::{ActiveModelTrait as _, EntityTrait as _},
};
use std::{
    env::{self, current_dir},
    process::Command,
};

use clap::Parser as _;
use color_eyre::{eyre::Result, owo_colors::OwoColorize as _};
use emergence_zk::{
    EmergenceDb, Kasten, Tag, Zettel, ZettelBuilder,
    entities::{self},
    entity::ActiveValue,
};

use crate::args::{CliArgs, Commands};

mod args;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = CliArgs::parse();

    match args.command {
        Commands::Init(args) => {
            Kasten::new(&args.name).await?;

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
            //TODO: yeah realistically we just have to make sure that the metadata for the kasten exists
            // let _: Kasten = Kasten::parse(&pwd)
            //     .inspect_err(|e| eprintln!("You arent in a valid kasten! {e}"))?;

            let db = EmergenceDb::connect(pwd.clone()).await?;

            let mut zb = ZettelBuilder::new(pwd);

            if let Some(name) = args.name {
                zb.name(name);
            }

            for tag in args.tags {
                zb.add_tag(Tag::new(tag, "penis")?);
            }

            let z: Zettel = zb.build(&db).await?;

            let editor = env::var("EDITOR")
                .or_else(|_| env::var("VISUAL"))
                .unwrap_or_else(|_| "vim".to_owned());

            Command::new(editor).arg(&z.path).status()?;

            Ok(())
        }

        Commands::List => {
            let pwd = current_dir()?;
            let db = EmergenceDb::connect(pwd).await?;

            let x = entities::zettel::Entity::find().all(db.as_ref()).await?;

            for zettel in x {
                println!("{zettel:#?}");
            }

            Ok(())
        }

        Commands::Test => {
            let pwd = current_dir()?;
            let db = EmergenceDb::connect(pwd).await?;

            println!("{db:#?}");

            let new_zettel = entities::zettel::ActiveModel {
                nanoid: ActiveValue::Set(ZettelId::default().to_string()),
                title: ActiveValue::Set("whateva".to_owned()),

                ..Default::default()
            };
            let zettel = new_zettel
                .insert(db.as_ref())
                .await
                .expect("inserting zk failed");

            let new_tag = tag::ActiveModel {
                name: ActiveValue::Set("penis_tag".to_owned()),
                nanoid: ActiveValue::Set(ZettelId::default().to_string()),
                color: ActiveValue::Set("dumb".to_owned()),
                ..Default::default()
            };

            let tag = new_tag
                .insert(db.as_ref())
                .await
                .expect("inserting tag failed");

            let tag_zettel_link = zettel_tag::ActiveModel {
                tag_nano_id: ActiveValue::Set(tag.nanoid),
                zettel_nano_id: ActiveValue::Set(zettel.nanoid),
            };

            let x = tag_zettel_link
                .insert(db.as_ref())
                .await
                .expect("should have inserted properly");

            let entities: Vec<zettel::Model> = zettel::Entity::find()
                .left_join(tag::Entity)
                .into_model()
                .all(db.as_ref())
                .await
                .expect("works?");

            println!("model: {x:?}");
            println!("entities: {entities:?}");

            Ok(())
        }
    }
}
