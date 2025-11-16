use crate::zkreator::ZKreator;

mod zkreator;

#[test]
fn test_basic() {
    let _creator = ZKreator::new(5, 5);
}
