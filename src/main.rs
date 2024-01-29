use std::string;

use reqwest::Error;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use tracing::{error, info};

use scraper::{Html, Selector};

async fn allkpop() -> Result<Vec<String>, Error> { 
    let resp = match reqwest::get("https://www.allkpop.com/?view=a&feed=a&sort=d").await {
        Ok(res) => res,
        Err(e) => {
            error!("Error on request to allkpop");
            return Err(e);
        }
    };
    let text = resp.text().await?;

    let document = Html::parse_document(&text);
    let article_selector = Selector::parse("article.list").unwrap();

    let mut list: Vec<String> = Vec::new();
    for each in document.select(&article_selector).take(10) {
        let a_selector = Selector::parse(r#"div.content>div>div.text>div.title>a"#).unwrap();
        for a_elem in each.select(&a_selector) {
            let mut url = "https://www.allkpop.com".to_owned();
            let href = a_elem.value().attr("href").expect("href not found");
            url.push_str(href); 
            list.push(href.to_string());
        }
    }
    Ok(list)
}


struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            }
        } else if msg.content == "!secret" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "say sorry").await {
                error!("Error sending message: {:?}", e);
            }
        } else if msg.content == "!allkpop" {
            let result = allkpop().await;
            match result {
                Ok(list) => {
                    for each in list {
                        if let Err(e) = msg.channel_id.say(&ctx.http, each.as_str()).await {
                            error!("Error sending message: {:?}", e);
                        }
                    }
                },
                Err(e) => {
                    error!("Error sending message: {:?}", e);
                }
            } 
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = secret_store.get("DISCORD_TOKEN").expect("Expected a token in the secrets file");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}

/*
#[shuttle_service::main]
async fn serenity(#[shared::Postgres] pool: PgPool) -> shuttle_service::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml` from the shared Postgres database


    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client)
}
*/

mod tests {
    use crate::allkpop;

    #[tokio::test]
    async fn test_allkpop() {
        println!("testing allkpop function");
        let result = allkpop().await;
        assert!(result.is_err())
    }
}
