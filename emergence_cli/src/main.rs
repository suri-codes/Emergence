use std::path::PathBuf;

use emergence_zk::{Tag, ZettelBuilder};

fn main() {
    let test_root = PathBuf::from("./test");

    let zk_b = ZettelBuilder::new(test_root);

    let _zk = zk_b
        .name("kill adrien!")
        .add_tag(Tag::new("test", "color!").expect("color"))
        .add_tag(Tag::new("death", "color!").expect("color"))
        .add_tag(Tag::new("mediatok", "color!").expect("color"))
        .content("Adrian is just so butt")
        .build()
        .expect("lol");
}
