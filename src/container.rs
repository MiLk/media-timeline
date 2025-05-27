use crate::domain::services::hashtag::SubscribedHashtagService;
use crate::domain::services::status::StatusService;
use crate::infrastructure::database::sqlite;
use crate::infrastructure::repositories::hashtag::SubscribedHashtagSqliteRepository;
use crate::infrastructure::repositories::status::{
    RecentStatusSqliteRepository, StatusSqliteRepository,
};
use crate::infrastructure::services::mastodon::MastodonClient;
use crate::infrastructure::services::templating;
use crate::services::hashtag::SubscribedHashtagServiceImpl;
use crate::services::status::StatusServiceImpl;
use actix_web::web;
use std::sync::Arc;
use tera::Tera;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Container {
    pub tera: Arc<Tera>,
    pub mastodon: Arc<MastodonClient>,
    pub status_service: Arc<dyn StatusService>,
    pub subscribed_hashtag_service: Arc<dyn SubscribedHashtagService>,
}

impl Container {
    pub async fn new() -> Self {
        let tera =
            templating::initialize_tera().expect("Unable to initialize templating engine Tera");

        let user_agent = Some(String::from(format!("{}/{}", PKG_NAME, PKG_VERSION)));
        let mastodon = Arc::new(
            MastodonClient::new("https://dice.camp".to_owned(), user_agent)
                .expect("Unable to initialize the Mastodon client"),
        );

        let pool = Arc::new(sqlite::new().expect("Unable to initialize the database connection"));
        let subscribed_hashtag_repository =
            Arc::new(SubscribedHashtagSqliteRepository::new(pool.clone()));
        let recent_status_repository = Arc::new(RecentStatusSqliteRepository::new(pool.clone()));
        let status_index_repository = Arc::new(StatusSqliteRepository::new(pool.clone()));

        let subscribed_hashtag_service = Arc::new(SubscribedHashtagServiceImpl::new(
            subscribed_hashtag_repository,
        ));
        let status_service = Arc::new(StatusServiceImpl::new(
            mastodon.clone(),
            recent_status_repository.clone(),
            status_index_repository.clone(),
        ));

        Container {
            tera: Arc::new(tera),
            mastodon,
            status_service,
            subscribed_hashtag_service,
        }
    }

    pub fn config(&self, cfg: &mut web::ServiceConfig) {
        cfg.app_data(web::Data::from(self.tera.clone()))
            .app_data(web::Data::from(self.mastodon.clone()))
            .app_data(web::Data::from(self.status_service.clone()))
            .app_data(web::Data::from(self.subscribed_hashtag_service.clone()));
    }
}
