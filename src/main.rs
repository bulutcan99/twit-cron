use anyhow::Result;
use axum::{extract::State, routing::post, Json, Router};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
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

// Tweet Gönderim Verisi
#[derive(Debug, Deserialize)]
struct TweetRequest {
    tweets: Vec<TweetData>,
}

// Her Tweet İçin Veriler
#[derive(Debug, Deserialize)]
struct TweetData {
    text: String,
    scheduled_at: NaiveDateTime, // ISO8601 formatında tarih beklenir
}

// Yanıt Formatı
#[derive(Debug, Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

// Auth Verilerini Saklamak İçin Struct
struct AppState {
    auth: OAuthAuthentication,
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
    let state = Arc::new(AppState { auth });

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

    let scheduler = match JobScheduler::new().await {
        Ok(scheduler) => scheduler,
        Err(_) => {
            return Json(ApiResponse {
                success: false,
                message: "Scheduler not setted!".to_string(),
            });
        }
    };

    let notify = Arc::new(Notify::new());
    let now = Utc::now() + Duration::hours(3);
    let now_naive = now.naive_utc();
    let job_size = payload.tweets.len();
    let job_remaining = Arc::new(Mutex::new(job_size));

    for (idx, tweet) in payload.tweets.iter().enumerate() {
        if tweet.scheduled_at < now_naive {
            return Json(ApiResponse {
                success: false,
                message: "Tweet gönderme zamanı geçmiş olamaz!".to_string(),
            });
        }

        let job_time = tweet.scheduled_at.format("%H:%M:%S").to_string();
        let cron_timer = format!("{} * * * * *", job_time); // Cron formatını doğru şekilde oluşturun
        let job_remaining = job_remaining.clone();
        let notify = notify.clone();

        // Tweet gönderme işlemi
        let body = post_2_tweets::Body {
            text: Some(tweet.text.clone()),
            ..Default::default()
        };

        let builder = post_2_tweets::Api::new(body).build(&state.auth);
        let tweet_remaining = job_remaining.clone();
        let notify = notify.clone();

        // Cron job ekleyerek tweet gönderimini zamanla
        if let Err(e) = scheduler.add(Job::new_async(cron_timer, move |uuid, mut l| {
            Box::pin(async move {
                // Tweet gönderme işlemi
                match execute_twitter::<serde_json::Value>(builder).await {
                    Ok((res, _)) => {
                        println!("Tweet başarıyla gönderildi: {:?}", res);
                    }
                    Err(e) => {
                        println!("Tweet gönderme hatası: {:?}", e);
                    }
                }

                // İş tamamlandıktan sonra job_remaining sayacını güncelle
                let mut remaining_jobs = tweet_remaining.lock().await;
                *remaining_jobs -= 1;

                // Tüm işler tamamlandığında notify tetikleyin
                if *remaining_jobs == 0 {
                    println!("Tüm işler tamamlandı.");
                    notify.notify_one(); // Tüm işler bittiğinde sinyal gönder
                }
            })
        })) {
            // Eğer job eklerken hata alırsak, logla ve devam et
            println!("Job eklenemedi: {:?}", e);
        }
    }

    // Kullanıcıya hemen planlama işleminin tamamlandığını bildir
    Json(ApiResponse {
        success: true,
        message: "Tweetler başarıyla planlandı!".to_string(),
    })
}
