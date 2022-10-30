use chrono::{DateTime, Utc};
use poise::serenity_prelude::Mentionable;
use poise::serenity_prelude::{self as sr, GuildId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use tokio::time::{sleep, Duration};

mod config;
use config::read_config;

mod file_loader;
use file_loader::{open_mute_roles, open_server_levels, open_spam_count, open_unmute_times};

mod commands;

use commands::moderation;
use moderation::kick_ban;
use moderation::manage_messages;
use moderation::muting;
use moderation::muting::UnmutedTime;

use commands::level_system::xp;

use crate::commands::level_system::xp::ServerMember;

#[derive(Deserialize, Serialize, Clone)]
pub struct SpamCount {
    amount_of_messages_without_change: i32,
    message_content: String,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, (), Error>;

async fn event_listener(
    ctx: &sr::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, (), Error>,
    _user_data: (),
) -> Result<(), Error> {
    let config = read_config().unwrap();
    match event {
        poise::Event::Ready { data_about_bot } => {
            println!("{} is connected!", data_about_bot.user.name);

            // ------------------------------------------------------------------------------------

            if config.level_system.levels_on == true {
                let mut levels_dict = open_server_levels();
                let mut global_levels = levels_dict.get("global").unwrap().clone();

                // setting everyone's can_gain_xp variable to true on startup
                for mut server in levels_dict.clone() {
                    for mut member in server.1.members.clone() {
                        member.1.can_gain_xp = true;
                        server.1.members.insert(member.0, member.1);
                    }
                    levels_dict.insert(server.0, server.1);
                }

                // setting every members global level and total_xp to 0 in prep for the next step
                for mut member in global_levels.members.clone() {
                    member.1.total_xp = 0;
                    member.1.level = 0;
                    global_levels.members.insert(member.0, member.1);
                }
                // calculating the level and total_xp for global levels
                for server in levels_dict.clone() {
                    if server.0 == "global" {
                        continue;
                    } else {
                        for member in server.1.members {
                            if !global_levels
                                .clone()
                                .members
                                .contains_key(&member.0.clone())
                            {
                                global_levels
                                    .members
                                    .insert(member.0.clone(), ServerMember::new());
                            }

                            let mut global_member =
                                global_levels.members.get(&member.0).unwrap().clone();
                            global_member.level += member.1.level;
                            global_member.total_xp += member.1.total_xp;
                            global_levels.members.insert(member.0, member.1);
                        }
                    }
                }

                let buf = Vec::new();
                let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
                let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
                levels_dict.serialize(&mut ser).unwrap();

                let mut levels =
                    fs::File::create("./src/commands/level_system/levels.json").unwrap();
                write!(levels, "{}", String::from_utf8(ser.into_inner()).unwrap()).unwrap();
            }

            // ------------------------------------------------------------------------------------
            // checking for people who need to be unmuted according to the unmuted_times.json file

            loop {
                sleep(Duration::from_millis(1000)).await;
                let now = Utc::now();

                let mut unmute_times_dict = open_unmute_times();

                for server in unmute_times_dict.clone() {
                    for member in server.1.unmuted_time {
                        let time_unmuted: DateTime<Utc> = member.1.parse().unwrap();

                        if now > time_unmuted {
                            let user_id = member.0.parse::<u64>().unwrap();
                            let guild_id = server.0.parse::<u64>().unwrap();

                            let mute_roles = open_mute_roles();

                            let mute_role = mute_roles
                                .get(&guild_id.to_string())
                                .unwrap()
                                .parse::<u64>()
                                .unwrap();

                            let guild = GuildId(guild_id);

                            let mut member_in_server =
                                guild.member(ctx.http.clone(), user_id).await?;

                            member_in_server
                                .remove_role(ctx.http.clone(), mute_role)
                                .await?;

                            let mut user_dict = unmute_times_dict
                                .get(&guild_id.to_string())
                                .unwrap()
                                .clone();

                            user_dict.unmuted_time.remove(&user_id.to_string());

                            unmute_times_dict
                                .insert(guild_id.to_string().clone(), user_dict.clone());

                            let buf = Vec::new();
                            let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
                            let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
                            unmute_times_dict.serialize(&mut ser).unwrap();

                            let mut unmute_times =
                                fs::File::create("./src/commands/moderation/unmuted_times.json")
                                    .unwrap();

                            write!(
                                unmute_times,
                                "{}",
                                String::from_utf8(ser.into_inner()).unwrap()
                            )
                            .unwrap();

                            println!("{}, has been unmuted!", member_in_server.user.name);
                        } else {
                            continue;
                        }
                    }
                }
            }
        }
        poise::Event::Message { new_message } => {
            // println!("in {} from {}: {}", new_message.guild(&_ctx.cache).unwrap().name, new_message.member(&_ctx.http).await.unwrap().display_name, new_message.content);
            let message = new_message.clone();
            let author_id = message.clone().author.id.to_string();
            let guild_id = message.clone().guild_id.unwrap().to_string();
            let perms = message
                .clone()
                .member(&ctx.http)
                .await
                .unwrap()
                .permissions(&ctx.cache)
                .unwrap();

            if config.spam_settings.antispam == true {
                // if the author has admin perms
                if perms.administrator() == true {
                    return Ok(());
                // if the author is a bot
                } else if message.clone().author.bot == true {
                    return Ok(());
                // if the author has manage_messages perms
                } else if perms.manage_messages() == true {
                    return Ok(());
                }
                // checking if the message contained any black listed words
                for black_listed_word in config.word_blacklist {
                    for word in message.content.split(" ") {
                        if word.to_string().contains(&black_listed_word) {
                            message.channel_id.send_message(&ctx.http, |m| {
                                m.content(format!("{} your message has been deleted because it contained a black-listed word.", message.author.mention())) 
                            }).await?;
                            return Ok(());
                        }
                    }
                }

                // spam checker
                let mut spam_count_dict = open_spam_count();

                let mut user_bmap = BTreeMap::new();

                user_bmap.insert(
                    author_id.clone(),
                    SpamCount {
                        amount_of_messages_without_change: 0,
                        message_content: message.clone().content.to_string(),
                    },
                );

                if !spam_count_dict.contains_key(&guild_id) {
                    spam_count_dict.insert(guild_id.clone(), user_bmap);
                }

                let mut spam_count = spam_count_dict.clone().get(&guild_id).unwrap().clone();

                if !spam_count.contains_key(&author_id.clone()) {
                    spam_count.insert(
                        author_id.clone(),
                        SpamCount {
                            amount_of_messages_without_change: 0,
                            message_content: message.clone().content.to_string(),
                        },
                    );
                }

                if spam_count.get(&author_id.clone()).unwrap().message_content
                    != message.clone().content
                {
                    spam_count.insert(
                        author_id.clone(),
                        SpamCount {
                            amount_of_messages_without_change: 0,
                            message_content: message.clone().content.to_string(),
                        },
                    );
                }

                if spam_count
                    .get(&author_id)
                    .unwrap()
                    .amount_of_messages_without_change
                    == 0
                {
                    spam_count.insert(
                        author_id.clone(),
                        SpamCount {
                            amount_of_messages_without_change: 1,
                            message_content: message.clone().content.to_string(),
                        },
                    );
                } else if spam_count
                    .get(&author_id)
                    .unwrap()
                    .amount_of_messages_without_change
                    >= config.spam_settings.spam_count - 1
                {
                    let mute_roles = open_mute_roles();

                    let mute_role = mute_roles
                        .get(&message.clone().guild_id.unwrap().to_string())
                        .unwrap()
                        .parse::<u64>()
                        .unwrap();

                    message
                        .clone()
                        .member(&ctx.http)
                        .await
                        .unwrap()
                        .add_role(&ctx.http, mute_role)
                        .await?;

                    message
                        .channel_id
                        .send_message(&ctx.http, |m| {
                            m.content(format!(
                                "{} you have been muted for spamming.",
                                message.author.mention()
                            ))
                        })
                        .await?;

                    spam_count.insert(
                        author_id.clone(),
                        SpamCount {
                            amount_of_messages_without_change: 0,
                            message_content: message.clone().content.to_string(),
                        },
                    );
                } else {
                    spam_count.insert(
                        author_id.clone(),
                        SpamCount {
                            amount_of_messages_without_change: spam_count
                                .get(&author_id.clone())
                                .unwrap()
                                .amount_of_messages_without_change
                                + 1,
                            message_content: message.clone().content.to_string(),
                        },
                    );
                }

                if spam_count.len() > 50 {
                    spam_count.pop_last();
                }

                spam_count_dict.insert(message.clone().guild_id.unwrap().to_string(), spam_count);

                let buf = Vec::new();
                let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
                let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
                spam_count_dict.serialize(&mut ser).unwrap();

                let mut spam_count_file =
                    fs::File::create("./src/commands/moderation/spam_count.json").unwrap();

                write!(
                    spam_count_file,
                    "{}",
                    String::from_utf8(ser.into_inner()).unwrap()
                )
                .unwrap();
            }
            if config.level_system.levels_on == true {
                // TODO
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
        muting::mute(),
        muting::unmute(),
        muting::muterole(),
        kick_ban::kick(),
        kick_ban::ban(),
        manage_messages::purge(),
        xp::check_xp(),
    ];

    let framework = poise::Framework::builder()
        .token(config.tokens.discord_token)
        .user_data_setup(|_, _, _| Box::pin(async move { Ok(()) }))
        .intents(sr::GatewayIntents::non_privileged() | sr::GatewayIntents::MESSAGE_CONTENT)
        .options(poise::FrameworkOptions {
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
