use actix_web::HttpServer;
use media_timeline::container::Container;
use media_timeline::create_app::create_app;
use media_timeline::workers::timeline::TimelineUpdater;
use media_timeline::workers::tracker::WorkerTracker;
use std::env;
use std::error::Error;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let container: Arc<Container> = Arc::new(Container::new().await);
    let mut workers = WorkerTracker::new();
    workers.register_worker(TimelineUpdater::new(container.clone()));
    workers.start();

    let listen_addr = env::var("LISTEN_ADDR").unwrap_or("127.0.0.1".to_owned());
    let server =
        HttpServer::new(move || create_app(container.clone())).bind((listen_addr, 1337))?;

    for (addr, scheme) in server.addrs_with_scheme() {
        println!("Listening on {}://{}", scheme, addr);
    }

    server.run().await?;

    workers.stop();

    workers.wait().await;

    Ok(())
}
