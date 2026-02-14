use actix_web::{get, web, App, HttpServer, HttpResponse};
use reqwest;
use serde::{Serialize, Deserialize};
use futures::future::join_all;

const HN_TOP_STORIES_URL: &str = "https://hacker-news.firebaseio.com/v0/topstories.json";
const HN_ITEM_URL: &str = "https://hacker-news.firebaseio.com/v0/item";

#[derive(Deserialize)]
struct TopStoriesQuery {
    top_n: Option<usize>
}

#[derive(Deserialize, Serialize)]
struct Story {
    // id: u64,
    title: Option<String>,
    url: Option<String>,
    // score: Option<i32>,
    // by: Option<String>,
    // time: Option<u64>,
}

#[get("/top-stories")]
async fn top_stories(query: web::Query<TopStoriesQuery>) -> Result<HttpResponse, actix_web::Error> {
    let client = reqwest::Client::new();
    let response: Vec<u64> = client
        .get(HN_TOP_STORIES_URL)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        .json()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let top_n = query.top_n.unwrap_or(5);
    // let stories: Vec<u64> = response.into_iter().take(top_n).collect();

    let story_futures = response
        .into_iter()
        .take(top_n)
        .map(|id| {
            let client = &client;
            async move {
                client
                    .get(format!("{}/{}.json", HN_ITEM_URL, id))
                    .send()
                    .await?
                    .json::<Story>()
                    .await
            }
        });
    let stories: Vec<Story> = join_all(story_futures)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    Ok(HttpResponse::Ok().json(stories))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(top_stories)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
