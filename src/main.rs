use std::sync::Arc;

use tokio::sync::Mutex;

use axum::{
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Extension, Router, Server,
};

use chrono::{Duration, Utc};
use icalendar::{Calendar, Class, Component, Event};

#[derive(Debug)]
struct CalendarState {
    calendar: Calendar,
}

type CalendarStateExtension = Extension<Arc<Mutex<CalendarState>>>;

#[tokio::main]
async fn main() {
    env_logger::init();
    let calendar = Calendar::new()
        .name("example calendar")
        .ttl(&Duration::minutes(1))
        .done();
    let shared_state = Arc::new(Mutex::new(CalendarState { calendar }));
    let app = Router::new().route("/", get(ical).layer(Extension(shared_state)));

    // run it with hyper on localhost:3000
    Server::bind(&"0.0.0.0:8888".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ical(Extension(state): CalendarStateExtension) -> impl IntoResponse {
    log::info!("Calendar requested");

    let mut state = state.lock().await;

    let id = state.calendar.len() + 1;
    let start = Utc::now() + Duration::hours(id as i64);
    let end = start + Duration::minutes(10 + id as i64);
    let summary = format!("test event {}", id);

    state.calendar.push(
        Event::new()
            .summary(&summary)
            .description("here I have something really important to do")
            .starts(start)
            .ends(end)
            .class(Class::Confidential)
            .status(icalendar::EventStatus::Confirmed)
            .add_property("SEQUENCE", "1")
            .done(),
    );

    let body = state.calendar.to_string();

    let mut headers = HeaderMap::new();
    headers.append(
        header::CONTENT_TYPE,
        "text/calendar; charset=utf-8".parse().unwrap(),
    );
    headers.append(
        header::CONTENT_DISPOSITION,
        "inline; filename=calendar.ics".parse().unwrap(),
    );

    (StatusCode::OK, headers, body)
}
