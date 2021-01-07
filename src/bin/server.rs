use std::iter;

use vatsim_stats::server;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    server::run(DummyDataProvider).await.unwrap();
}

struct DummyDataProvider;

impl server::DataProvider for DummyDataProvider {
    fn data_since(
        &self,
        _time: chrono::DateTime<chrono::Utc>,
    ) -> Box<dyn Iterator<Item = vatsim_stats::datafeed::Datafeed>> {
        Box::new(iter::empty())
    }
}
