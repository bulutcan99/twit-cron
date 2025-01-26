use dotenv::dotenv;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client,
};
use serde::Serialize;
use std::env;
use tokio_cron_scheduler::{Job, JobScheduler};

#[derive(oauth::Request, Serialize)]
struct Tweet {
    text: String,
}

async fn send_tweet(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Twitter API anahtarlarını yükle
    let api_key = env::var("TWITTER_API_KEY")?;
    let api_secret_key = env::var("TWITTER_API_SECRET_KEY")?;
    let access_token = env::var("TWITTER_ACCESS_TOKEN")?;
    let access_token_secret = env::var("TWITTER_ACCESS_TOKEN_SECRET")?;

    // Twitter API endpointi
    let uri = "https://api.twitter.com/2/tweets";

    // Authorization header'ını oluştur
    let request = Tweet {
        text: text.to_string(),
    };

    println!("API_KEY: {:?}", api_key.as_str());
    println!("API_SECRET_KEY: {:?}", api_secret_key.as_str());
    println!("ACCESS_KEY: {:?}", access_token.as_str());
    println!("ACCESS SECRET: {:?}", access_token_secret.as_str());

    let token = oauth::Token::from_parts(
        api_key.as_str(),
        api_secret_key.as_str(),
        access_token.as_str(),
        access_token_secret.as_str(),
    );

    let authorization_header = oauth::post(uri, &request, &token, oauth::HMAC_SHA1);
    let form = oauth::to_form(&request);
    let uri = oauth::to_query(uri.to_owned(), &request);

    let client = Client::new();
    let res = client.post(uri).json(&request).send().await?;

    if res.status().is_success() {
        println!("Tweet başarıyla gönderildi!");
    } else {
        println!("Tweet gönderilirken hata oluştu: {:?}", res.text().await?);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Çevre değişkenlerini yükle
    dotenv().ok();

    // Cron Scheduler oluştur
    let scheduler = JobScheduler::new().await?;

    // Cron Job tanımla (1 saatte bir çalışacak)
    // Bu ifade her saat başında (örn. 12:00, 13:00, 14:00) job'u çalıştırır.
    let job = Job::new_async("0 0/1 * * * *", |_uuid, _lock| {
        Box::pin(async {
            println!("Tweet atılıyor (1 saatlik cron)...");
            if let Err(e) = send_tweet("Bu bir cron job ile atılan tweet! #Rust").await {
                eprintln!("Tweet gönderilirken bir hata oluştu: {}", e);
            }
        })
    })?;

    // Scheduler'a cron job ekle
    scheduler.add(job).await?;

    // İlk tweet'i hemen at
    println!("Tweet atılıyor (ilk tweet)...");
    if let Err(e) = send_tweet("Cron deneme").await {
        eprintln!("Tweet gönderilirken bir hata oluştu: {}", e);
    }

    // Scheduler'ı başlat
    scheduler.start().await?;

    // Scheduler'ın çalışmasını sonsuza kadar bekle
    tokio::signal::ctrl_c().await?;
    println!("Uygulama sonlandırıldı.");
    Ok(())
}
