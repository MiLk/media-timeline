use actix_settings::{ApplySettings, BasicSettings, Mode};
use actix_web::HttpServer;
use media_timeline::container::Container;
use media_timeline::create_app::create_app;
use media_timeline::settings::ApplicationSettings;
use media_timeline::workers::timeline::TimelineUpdater;
use media_timeline::workers::tracker::WorkerTracker;
use std::error::Error;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut settings: BasicSettings<ApplicationSettings> =
        BasicSettings::parse_toml("./config.toml")
            .expect("Failed to parse `Settings` from config.toml");

    BasicSettings::<ApplicationSettings>::override_field_with_env_var(
        &mut settings.actix.hosts,
        "ACTIX_HOSTS",
    )?;
    BasicSettings::<ApplicationSettings>::override_field_with_env_var(
        &mut settings.actix.mode,
        "ACTIX_MODE",
    )?;

    init_logger(&settings);

    let container: Arc<Container> = Arc::new(Container::new(settings.clone()).await);
    let mut workers = WorkerTracker::new();
    workers.register_worker(TimelineUpdater::new(container.clone()));
    workers.start();

    let server =
        HttpServer::new(move || create_app(container.clone())).try_apply_settings(&settings)?;

    for (addr, scheme) in server.addrs_with_scheme() {
        println!("Listening on {}://{}", scheme, addr);
    }

    server.run().await?;

    workers.stop();

    workers.wait().await;

    Ok(())
}

/// Initialize the logging infrastructure.
fn init_logger(settings: &BasicSettings<ApplicationSettings>) {
    if !settings.actix.enable_log {
        return;
    }

    unsafe {
        std::env::set_var(
            "RUST_LOG",
            match settings.actix.mode {
                Mode::Development => "debug,actix_web=debug,media_timeline=debug",
                Mode::Production => "info,actix_web=info,media_timeline=debug",
            },
        );

        std::env::set_var("RUST_BACKTRACE", "1");
    }

    env_logger::init();
}
