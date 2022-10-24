use crate::{read_config, Context, Error};
use poise::serenity_prelude as sr;
use poise::serenity_prelude::CacheHttp;
use poise::serenity_prelude::Mentionable;
use sr::RoleId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use chrono::{Utc, Duration};
use tokio::time::{Duration as TokioDuration, sleep};

// ---------PURGE COMMAND--------------------------------------------------------------------------------------

#[poise::command(prefix_command, aliases("clear"))]
pub async fn purge(
    ctx: Context<'_>,
    #[description = "Number of messages to delete."]
    num_mesg_to_delete: u64
) -> Result<(), Error> {
    
    let channel_id = ctx.channel_id();
    let message_id = ctx.id();
    
    let messages = channel_id
        .messages(&ctx.discord().http, |retriever| retriever.before(message_id).limit(num_mesg_to_delete))
        .await?;
    
    channel_id.delete_messages(ctx.discord().http.clone(), messages).await?;
            
    ctx.discord().http.delete_message(channel_id.0, message_id).await?;
    let message = ctx.say(format!("{} messages have been deleted!", num_mesg_to_delete)).await?;
    sleep(TokioDuration::from_millis(5000)).await;
    message.delete(ctx).await?;
    
    
    Ok(())
}