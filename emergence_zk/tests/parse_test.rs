use crate::zkreator::ZKreator;

mod zkreator;

#[tokio::test]
async fn test_basic() {
    let creator = ZKreator::new(2000, 5000);

    creator.create().await.expect("zkreator is perfect bro.");
}
