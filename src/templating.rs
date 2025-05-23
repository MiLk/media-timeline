use chrono::{DateTime, Utc};
use serde_json::to_value;
use std::collections::HashMap;
use std::ops::Sub;
use tera::{try_get_value, Tera};

fn timedelta_filter(
    value: &tera::Value,
    _args: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let v = try_get_value!("timedelta_filter", "value", String, value);
    let datetime = DateTime::parse_from_rfc3339(v.as_str())
        .map_err(|err| tera::Error::from(err.to_string()))?;
    let delta = Utc::now().with_timezone(datetime.offset()).sub(datetime);
    if delta.num_days() > 0 {
        return Ok(to_value(format!("{}d", delta.num_days()))?);
    }
    if delta.num_hours() > 0 {
        return Ok(to_value(format!("{}h", delta.num_hours()))?);
    }
    Ok(to_value(format!("{}m", delta.num_minutes()))?)
}

pub fn init_tera() -> tera::Result<Tera> {
    let mut tera = Tera::new("templates/**/*.html")?;
    tera.register_filter("timedelta", timedelta_filter);
    Ok(tera)
}
