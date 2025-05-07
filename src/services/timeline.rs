use crate::mastodon::MastodonClient;
use crate::storage::Storage;
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

async fn paginate_timeline(
    storage: web::Data<Storage>,
    mastodon_client: web::Data<MastodonClient>,
    hashtag: String,
) -> Result<Vec<Status>, megalodon::error::Error> {
    match storage.get_recent_status_id(hashtag.as_str()) {
        None => {
            let statuses = mastodon_client.get_tag_timeline(&hashtag, None).await?;
            if let Some(status) = statuses.last() {
                storage.set_recent_status_id(hashtag.clone(), status.id.clone());
            }
            Ok(statuses)
        }
        Some(recent_id) => {
            let mut statuses: Vec<Status> = vec![];
            let mut last_id = recent_id.clone();
            loop {
                let page = mastodon_client
                    .get_tag_timeline(&hashtag, Some(last_id))
                    .await?;
                if page.is_empty() {
                    break;
                }
                match page.last() {
                    None => break, // Should already be covered by checking if the page is empty
                    Some(status) => last_id = status.id.clone(),
                }
                debug!("Retrieved {} new statuses for {} - last: {}", page.len(), hashtag, last_id);
                storage.set_recent_status_id(hashtag.clone(), last_id.clone());
                statuses.extend(page)
            }
            statuses.sort_by_key(|status| Reverse((status.id.len(), status.id.clone())));
            Ok(statuses)
        }
    }
}

#[get("")]
async fn get_timeline(
    storage: web::Data<Storage>,
    tmpl: web::Data<Tera>,
    mastodon_client: web::Data<MastodonClient>,
) -> Result<impl Responder, error::Error> {
    let hashtags = storage.list_hashtags();
    let mut statuses = vec![];

    let mut tasks = HashMap::with_capacity(hashtags.len());
    for hashtag in &hashtags {
        let task = tokio::spawn(paginate_timeline(
            storage.clone(),
            mastodon_client.clone(),
            hashtag.clone(),
        ));
        tasks.insert(hashtag.clone(), task);
    }

    for (hashtag, task) in tasks {
        let h_statuses = task
            .await
            .map_err(error::ErrorInternalServerError)?
            .map_err(error::ErrorInternalServerError)?;
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

    storage.persist_statuses(&statuses).await?;

    let known_ids: HashSet<String> = statuses.iter().map(|s| s.id.clone()).collect();

    for status in storage.retrieve_statuses(&hashtags).await? {
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
