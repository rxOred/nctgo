use reqwest::Error;
use serenity::async_trait;
use serenity::builder::{CreateAttachment, CreateMessage};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use tracing::{error, info};

use scraper::{Html, Selector};

async fn marklee_twitter_summer_youth() {}

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
    for each in document.select(&article_selector).take(5) {
        let a_selector = Selector::parse(r#"div.content>div>div.text>div.title>a"#).unwrap();
        for a_elem in each.select(&a_selector) {
            let mut url = "https://www.allkpop.com".to_owned();
            let href = a_elem.value().attr("href").expect("href not found");
            url.push_str(href);
            list.push(url.to_string());
        }
    }
    Ok(list)
}

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            if let Err(e) = msg
                .channel_id
                .say(&ctx.http, "Hello ill get u markleee")
                .await
            {
                error!("Error sending message: {:?}", e);
            }
        } else if msg.content == "!say it" {
            match tokio::fs::File::open("say_sorry.jpg").await {
                Ok(f) => {
                    match CreateAttachment::file(&f, "say_sorry.jpg").await {
                        Ok(file) => {
                            let response = CreateMessage::new()
                                .content("Ever since the first day of meeting u, u brought me happiness but i only paid back to u with pressure and questions u could not answer.I tried to attribute this to the situation but now come to think of it, it is not entirely that. It has a lot to do with me. I tried to deal with the situation the best way i could when we first started facing it and i did to some extend. But at one point my emotions, feelings for u and insecurities I had got the best of me. So i couldn't see things clearly and could not understood somethings. We both lacked communication and i assumed a lot of things and didn't trust in you. So i myself take responsibility for the damage I caused to u and myself by ruining the chance to have a healthy relationship with u. For one last time I ask for your forgiveness. I am very sorry for my behaviour towards u and ur feelings.")
                                .add_file(file);
                            if let Err(e) = msg.channel_id.send_message(&ctx.http, response).await {
                                error!("Error sending message {:?}", e);
                            }
                        }
                        Err(e) => {
                            error!("Error sending message {:?}", e);
                        }
                    };
                }
                Err(e) => {
                    error!("Error sending message {:?}", e);
                }
            };
        } else if msg.content == "!allkpop" {
            let result = allkpop().await;
            match result {
                Ok(list) => {
                    if let Err(e) = msg
                        .channel_id
                        .say(&ctx.http, "Most recent news from AllKpop.com!")
                        .await
                    {
                        error!("Error sending message {:?}", e);
                    }
                    for each in list {
                        if let Err(e) = msg.channel_id.say(&ctx.http, each.as_str()).await {
                            error!("Error sending message: {:?}", e);
                        }
                    }
                }
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
    let token = secret_store
        .get("DISCORD_TOKEN")
        .expect("Expected a token in the secrets file");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}

mod tests {
    use crate::allkpop;

    #[tokio::test]
    async fn test_allkpop() {
        println!("testing allkpop function");
        let result = allkpop().await;
        assert!(result.is_err())
    }
}
