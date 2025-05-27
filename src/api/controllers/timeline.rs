use crate::domain::services::hashtag::SubscribedHashtagService;
use crate::domain::services::status::StatusService;
use actix_web::web::Html;
use actix_web::{Responder, error, get, web};
use log::debug;
use megalodon::entities::Status;
use serde::Serialize;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use tera::{Context, Tera};

#[derive(Serialize)]
struct TimelineContext {
    statuses: Vec<Status>,
}

#[get("")]
async fn get_timeline(
    subscribed_hashtag_service: web::Data<dyn SubscribedHashtagService>,
    status_service: web::Data<dyn StatusService>,
    tmpl: web::Data<Tera>,
) -> Result<impl Responder, error::Error> {
    let hashtags = subscribed_hashtag_service.list_hashtags()?;

    let mut tasks = HashMap::with_capacity(hashtags.len());
    for hashtag in &hashtags {
        let svc = status_service.clone();
        let hashtag_ = hashtag.clone();
        let task = tokio::spawn(async move { svc.paginate_timeline(hashtag_).await });
        tasks.insert(hashtag.clone(), task);
    }

    let mut statuses = vec![];
    for (hashtag, task) in tasks {
        let h_statuses = task.await.map_err(error::ErrorInternalServerError)?;
        debug!(
            "Retrieved {} statuses for tag {}",
            h_statuses.len(),
            &hashtag
        );
        statuses.extend(h_statuses);
    }

    // https://docs.joinmastodon.org/api/guidelines/#id
    statuses.sort_by_key(|status| Reverse((status.id.len(), status.id.clone())));
    statuses.dedup_by_key(|status| status.id.clone());

    debug!("{} statuses after deduplication", statuses.len());

    status_service.persist_statuses(&statuses).await?;

    let known_ids: HashSet<String> = statuses.iter().map(|s| s.id.clone()).collect();

    for status in status_service.retrieve_statuses(Some(&hashtags)).await? {
        if known_ids.contains(status.id.as_str()) {
            continue;
        }
        statuses.push(status);
    }

    statuses.sort_by_key(|status| Reverse(status.created_at));

    debug!("{} statuses after retrieving from storage", statuses.len());

    let timeline_context = TimelineContext { statuses };
    Context::from_serialize(timeline_context)
        .and_then(|context| tmpl.render("timeline.html", &context))
        .map(|rendered| Html::new(rendered))
        .map_err(error::ErrorInternalServerError)
}

pub fn timeline_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/timeline").service(get_timeline));
}
