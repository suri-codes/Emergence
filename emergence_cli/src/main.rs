use std::path::PathBuf;

use emergence_zk::{Kasten, Tag, Zettel, ZettelBuilder};

fn main() {
    let test_root = PathBuf::from("./test");

    let zk_b = ZettelBuilder::new(test_root.clone());

    let zk = zk_b
        .name("kill adrien!")
        .add_tag(Tag::new("test", "color!").expect("color"))
        .add_tag(Tag::new("death", "color!").expect("color"))
        .add_tag(Tag::new("mediatok", "color!").expect("color"))
        .content("Adrian is just so butt, [PENIS!](./cSRPIvjBQJfv6gjSthYD5.md)")
        .build()
        .expect("lol");

    let _zettel: Zettel = zk.path.as_path().try_into().expect("lol");

    let kasten = Kasten::generate(test_root).expect("Whateva");

    println!("Kasten: {kasten:#?}");
}
