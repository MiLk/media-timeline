use actix_web::web::Html;
use actix_web::{error, get, web, Responder};
use tera::{Context, Tera};

#[get("")]
async fn list_tags(tmpl: web::Data<Tera>) -> Result<impl Responder, error::Error> {
    let mut context = Context::new();
    let mut hashtags: Vec<String> = Vec::new();
    hashtags.push("HobbyStreak".to_owned());
    hashtags.push("PaintingMiniatures".to_owned());
    context.insert("hashtags", &hashtags);
    Ok(Html::new(
        tmpl.render("hashtags/list.html", &context)
            .map_err(|e| error::ErrorInternalServerError(e))?,
    ))
}

pub fn hashtags_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/tags").service(list_tags));
}
