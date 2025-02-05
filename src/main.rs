use anyhow::Result;
use axum::{extract::State, routing::post, Json, Router};
use chrono::{DateTime, Datelike, Duration, NaiveDateTime, Timelike, Utc};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::{
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::Notify;
use tokio_cron_scheduler::{Job, JobScheduler};
use twapi_v2::{
    api::{execute_twitter, post_2_tweets},
    oauth10a::OAuthAuthentication,
};

#[derive(Debug, Deserialize)]
struct TweetRequest {
    tweets: Vec<TweetData>,
}

#[derive(Debug, Deserialize)]
struct TweetData {
    text: String,
    scheduled_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

struct AppState {
    auth: OAuthAuthentication,
    notifier: Notify,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let api_key = env::var("TWITTER_API_KEY").expect("TWITTER_API_KEY not set");
    let api_secret_key =
        env::var("TWITTER_API_SECRET_KEY").expect("TWITTER_API_SECRET_KEY not set");
    let access_token = env::var("TWITTER_ACCESS_TOKEN").expect("TWITTER_ACCESS_TOKEN not set");
    let access_token_secret =
        env::var("TWITTER_ACCESS_TOKEN_SECRET").expect("TWITTER_ACCESS_TOKEN_SECRET not set");

    let auth = OAuthAuthentication::new(api_key, api_secret_key, access_token, access_token_secret);
    let notifier = Notify::new();
    let state = Arc::new(AppState { auth, notifier });

    let state_clone = state.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = state_clone.notifier.notified() => {
                println!("Tüm işler tamamlandı, server kapatılıyor.");
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Kullanıcı tarafından kapatma sinyali alındı.");

            }
        }
        std::process::exit(0);
    });

    let app = Router::new()
        .route("/tweet", post(handle_tweet))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Server running on http://{}", addr);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn handle_tweet(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TweetRequest>,
) -> Json<ApiResponse> {
    if payload.tweets.is_empty() || payload.tweets.len() > 20 {
        return Json(ApiResponse {
            success: false,
            message: "Tweet sayısı 1 ile 20 arasında olmalı!".to_string(),
        });
    }

    let now = Utc::now() + Duration::hours(3);
    let now_naive = now.naive_utc();
    let job_size = payload.tweets.len();
    let job_remaining = Arc::new(tokio::sync::Mutex::new(job_size));

    for (idx, tweet) in payload.tweets.iter().enumerate() {
        if tweet.scheduled_at < now_naive {
            return Json(ApiResponse {
                success: false,
                message: "Tweet gönderme zamanı geçmiş olamaz!".to_string(),
            });
        }

        let delay_seconds = (tweet.scheduled_at - now_naive).num_seconds();

        let state_clone = state.clone();
        let job_remaining = job_remaining.clone();
        let body = post_2_tweets::Body {
            text: Some(tweet.text.clone()),
            ..Default::default()
        };

        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(delay_seconds as u64)).await;
            let builder = post_2_tweets::Api::new(body).build(&state_clone.auth);
            match execute_twitter::<serde_json::Value>(builder).await {
                Ok((res, _)) => {
                    println!("Tweet {:?} başarıyla gönderildi: {:?}", idx, res);
                }
                Err(e) => {
                    println!("Tweet gönderme hatası: {:?}", e);
                }
            }

            let mut remaining_jobs = job_remaining.lock().await;
            *remaining_jobs -= 1;

            if *remaining_jobs == 0 {
                state_clone.notifier.notify_one();
            }
        });
    }

    Json(ApiResponse {
        success: true,
        message: "Tweetler başarıyla planlandı!".to_string(),
    })
}

fn convert_to_cron_format(date_time: NaiveDateTime) -> String {
    format!(
        "{} {} {} {} *",
        date_time.minute(),
        date_time.hour(),
        date_time.day(),
        date_time.month()
    )
}
