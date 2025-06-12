use crate::domain::services::hashtag::SubscribedHashtagService;
use crate::domain::services::status::StatusService;
use crate::settings::ApplicationSettings;
use actix_web::http::header::HttpDate;
use actix_web::http::{StatusCode, header};
use actix_web::web::Html;
use actix_web::{Either, HttpRequest, HttpResponse, Responder, error, get, web};
use chrono::Utc;
use log::debug;
use megalodon::entities::Status;
use serde::Serialize;
use std::cmp::Reverse;
use std::str::FromStr;
use std::time::SystemTime;
use tera::{Context, Tera};

#[derive(Serialize)]
struct TimelineContext {
    statuses: Vec<Status>,
}

#[get("")]
async fn get_timeline(
    request: HttpRequest,
    subscribed_hashtag_service: web::Data<dyn SubscribedHashtagService>,
    status_service: web::Data<dyn StatusService>,
    tmpl: web::Data<Tera>,
    settings: web::Data<ApplicationSettings>,
) -> Result<impl Responder, error::Error> {
    let hashtags = subscribed_hashtag_service.list_hashtags()?;

    let mut statuses = status_service
        .retrieve_statuses(Some(&hashtags), settings.timeline_statuses_count)
        .await?;
    statuses.sort_by_key(|status| Reverse(status.created_at));

    debug!("{} statuses retrieved from storage", statuses.len());

    let most_recent_dt = statuses
        .get(0)
        .map(|s| s.created_at)
        .unwrap_or_else(|| Utc::now());
    let most_recent: HttpDate = Into::<SystemTime>::into(most_recent_dt).into();

    let if_modified_since = request
        .headers()
        .get(header::IF_MODIFIED_SINCE)
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| HttpDate::from_str(s).ok());

    // Return 204 while If-Modified-Since is greater or equal to the most recent status
    if if_modified_since
        .map(|v| v.ge(&most_recent))
        .unwrap_or(false)
    {
        return Ok(Either::Left(
            HttpResponse::new(StatusCode::NOT_MODIFIED).customize(),
        ));
    }

    let timeline_context = TimelineContext { statuses };
    let res = Context::from_serialize(timeline_context)
        .and_then(|context| tmpl.render("timeline.html", &context))
        .map(|rendered| {
            Html::new(rendered)
                .customize()
                .append_header(header::CacheControl(vec![
                    header::CacheDirective::Private,
                    header::CacheDirective::MaxAge(
                        settings
                            .timeline_update_frequency()
                            .as_secs()
                            .try_into()
                            .unwrap_or(300),
                    ),
                    header::CacheDirective::Extension(
                        "stale-while-revalidate".to_string(),
                        Some("120".to_string()),
                    ),
                ]))
                .append_header(header::LastModified(most_recent))
        })
        .map_err(error::ErrorInternalServerError);
    Ok(Either::Right(res))
}

pub fn timeline_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/timeline").service(get_timeline));
}
