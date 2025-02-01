use anyhow::Result;
use chrono::prelude::*;
use dotenv::dotenv;
use std::env;
use twapi_v2::{
    api::{execute_twitter, post_2_tweets},
    oauth10a::OAuthAuthentication,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let api_key = env::var("TWITTER_API_KEY").expect("TWITTER_API_KEY not set");
    let api_secret_key =
        env::var("TWITTER_API_SECRET_KEY").expect("TWITTER_API_SECRET_KEY not set");
    let access_token = env::var("TWITTER_ACCESS_TOKEN").expect("TWITTER_ACCESS_TOKEN not set");
    let access_token_secret =
        env::var("TWITTER_ACCESS_TOKEN_SECRET").expect("TWITTER_ACCESS_TOKEN_SECRET not set");

    println!(
        "{:?}, {:?}, {:?}, {:?}",
        api_key.as_str(),
        api_secret_key.as_str(),
        access_token.as_str(),
        access_token_secret.as_str()
    );
    let auth = OAuthAuthentication::new(api_key, api_secret_key, access_token, access_token_secret);

    let now = Utc::now();
    let body = post_2_tweets::Body {
        text: Some(format!("now! {}, {}", now, "SAAA")),
        ..Default::default()
    };
    let builder = post_2_tweets::Api::new(body).build(&auth);
    let (res, _rate_limit) = execute_twitter::<serde_json::Value>(builder).await?;

    println!("{}", serde_json::to_string(&res).unwrap());

    let response = serde_json::from_value::<post_2_tweets::Response>(res)?;
    assert_eq!(response.is_empty_extra(), true);

    Ok(())
}
