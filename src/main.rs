use poise::serenity_prelude::{self as sr, GuildId};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use chrono::{Utc, DateTime};
use serde::Serialize;
use tokio::time::{Duration, sleep};

mod config;
use config::read_config;

mod commands;
use commands::moderation::kick_ban;
use commands::moderation::manage_messages;
use commands::moderation::muting;
use commands::moderation::muting::UnmutedTime;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, (), Error>;

async fn event_listener(
    _ctx: &sr::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, (), Error>,
    _user_data: (),
) -> Result<(), Error> {
    match event {
        poise::Event::Ready { data_about_bot } => {
            println!("{} is connected!", data_about_bot.user.name);
            
            loop {
                sleep(Duration::from_millis(1000)).await;
                let now = Utc::now();
                let mut unmute_times = fs::read_to_string("./src/commands/Moderation/unmuted_times.json").unwrap();
                if unmute_times == "" {
                    unmute_times = "{}".to_string(); 
                }
                let unmute_value: serde_json::Value = serde_json::from_str(&unmute_times).unwrap();
                let unmute_json = unmute_value.to_string();
                let mut unmute_times_dict = serde_json::from_str::<BTreeMap<String, UnmutedTime>>(&unmute_json).unwrap();
                for server in unmute_times_dict.clone() {
                    for member in server.1.unmuted_time {
                        let time_unmuted: DateTime<Utc> = member.1.parse().unwrap();
                        if now > time_unmuted {
                            let user_id  = member.0.parse::<u64>().unwrap();
                            let guild_id = server.0.parse::<u64>().unwrap();
                            
                            let mut mute_roles_file = fs::read_to_string("./src/commands/Moderation/mute_roles.json").unwrap();
                            if mute_roles_file == "" {
                                mute_roles_file = "{}".to_string();
                            }
                            let mute_roles_json: serde_json::Value = serde_json::from_str(&mute_roles_file).unwrap();
                            let mute_roles_json = mute_roles_json.to_string();
                            let mute_roles = serde_json::from_str::<BTreeMap<String, String>>(&mute_roles_json)?;
                            
                            let mute_role = mute_roles.get(&guild_id.to_string()).unwrap().parse::<u64>().unwrap();
                            
                            let guild = GuildId(guild_id);
                            let mut member_in_server =  guild.member(_ctx.http.clone(), user_id).await?;
                            
                            member_in_server.remove_role(_ctx.http.clone(),mute_role).await?;
                            let mut user_dict = unmute_times_dict.get(&guild_id.to_string()).unwrap().clone();
                            user_dict.unmuted_time.remove(&user_id.to_string());
                            unmute_times_dict.insert(guild_id.to_string().clone(), user_dict.clone());

                            // write to file
                            let buf = Vec::new();
                            let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
                            let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
                            unmute_times_dict.serialize(&mut ser).unwrap();
                            let mut unmute_times = fs::File::create("./src/commands/Moderation/unmuted_times.json").unwrap();
                            write!(unmute_times, "{}", String::from_utf8(ser.into_inner()).unwrap()).unwrap();
                            
                            println!("{}, has been unmuted!", member_in_server.user.name);
                        } else {
                            continue
                        }
                    }
                }
            } 
         }
        _ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let config = read_config().unwrap();

    let functions = vec![
        moderation::mute(),
        moderation::unmute(),
        moderation::muterole(),
        moderation::kick(),
        moderation::ban(),
        moderation::purge()
    ];

    let framework = poise::Framework::builder()
        .token(config.tokens.discord_token)
        .user_data_setup(|_, _, _| Box::pin(async move { Ok(()) }))
        .intents(sr::GatewayIntents::non_privileged() | sr::GatewayIntents::MESSAGE_CONTENT)
        .options(poise::FrameworkOptions {
            // This is also where commands go
            commands: functions,
            listener: |ctx, event, framework, user_data| {
                Box::pin(event_listener(ctx, event, framework, *user_data))
            },
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(config.prefix.into()),

                edit_tracker: Some(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(3600),
                )),

                case_insensitive_commands: true,
                
                ..Default::default()
            },

            ..Default::default()
        });
    framework.run().await.unwrap();
}
