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

// ---------KICK COMMAND---------------------------------------------------------------------------------------

#[poise::command(prefix_command)]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "Member to kick."]
    member: sr::Member,
    #[description = "Reason to kick member. (You can leave this blank.)"]
    reason: String
) -> Result<(), Error> {
    
    if reason == "" {
        member.kick(&ctx.discord().http).await?;
        ctx.say(format!("{} has been kicked.", member.user.mention())).await?;
    } else {
        member.kick_with_reason(&ctx.discord().http, &reason).await?;
        ctx.say(format!("{} has been kicked for {}.", member.user.mention(), &reason)).await?;
    }
    
    Ok(())
}

// ---------BAN COMMAND----------------------------------------------------------------------------------------

#[poise::command(prefix_command)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "Member to ban."]
    member: sr::Member,
    #[description = "Reason to ban member. (You can leave this blank.)"]
    reason: String
) -> Result<(), Error> {
    
    if reason == "" {
        member.ban(&ctx.discord().http, 0).await?;
        ctx.say(format!("{} has been banned.", member.user.mention())).await?;
    } else {
        member.ban_with_reason(&ctx.discord().http, 0, &reason).await?;
        ctx.say(format!("{} has been banned for {}.", member.user.mention(), &reason)).await?;
    }
    
    Ok(())
}


