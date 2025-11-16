use crate::zkreator::ZKreator;

mod zkreator;

#[tokio::test]
async fn test_basic() {
    let creator = ZKreator::new(1000, 10000);

    creator.create().await;
}
