use crate::{Context, Error};
use poise::serenity_prelude as sr;
use songbird::input::Restartable;
use sr::CreateEmbed;
use std::collections::HashMap;

#[poise::command(prefix_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "What song do you want to play?"] song: Vec<String>,
) -> Result<(), Error> {
    //todo!("check if the server_id is already in the SESSION_LIST and if it isnt add it ");
    let guild = ctx.guild().unwrap();
    let guild_id = ctx.clone().guild_id().unwrap();

    let song = song.join(" ");

    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            ctx.say("Not in a voice channel").await.unwrap();
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.discord())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let _ = manager.join(guild_id, connect_to).await;

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        if song.starts_with("https://") {
            if song.starts_with("https://www.youtube.com/playlist") {
            } else {
                let source = match Restartable::ytdl(song, true).await {
                    Ok(source) => source,
                    Err(why) => {
                        println!("Err starting source: {:?}", why);

                        ctx.channel_id()
                            .say(&ctx.discord().http, "Error sourcing ffmpeg")
                            .await?;

                        return Ok(());
                    }
                };
                handler.enqueue_source(source.into());
            }
        } else {
            let source = match Restartable::ytdl_search(song, true).await {
                Ok(source) => source,
                Err(why) => {
                    println!("Err starting source: {:?}", why);

                    ctx.channel_id()
                        .say(&ctx.discord().http, "Error sourcing ffmpeg")
                        .await?;

                    return Ok(());
                }
            };
            handler.enqueue_source(source.into());
        }

        ctx.say(format!(
            r#"Added "{}" to the queue, at position: #{}."#,
            handler
                .queue()
                .current_queue()
                .last()
                .unwrap()
                .metadata()
                .clone()
                .title
                .unwrap(),
            handler.queue().len()
        ))
        .await?;
    }

    Ok(())
}

#[poise::command(prefix_command)]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = ctx.clone().guild_id().unwrap();

    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            ctx.say("Not in a voice channel").await.unwrap();
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.discord())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let _ = manager.join(guild_id, connect_to).await;

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        handler.queue().pause().unwrap();

        ctx.say("The player has been paused.").await?;
    }

    Ok(())
}

#[poise::command(prefix_command, aliases("resume"))]
pub async fn unpause(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = ctx.clone().guild_id().unwrap();

    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            ctx.say("Not in a voice channel").await.unwrap();
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.discord())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let _ = manager.join(guild_id, connect_to).await;

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        handler.queue().resume().unwrap();

        ctx.say("The player has been un-paused.").await?;
    }

    Ok(())
}

#[poise::command(prefix_command, aliases("q"))]
pub async fn queue(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = ctx.clone().guild_id().unwrap();

    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            ctx.say("Not in a voice channel").await.unwrap();
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.discord())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let _ = manager.join(guild_id, connect_to).await;

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        let mut embed = CreateEmbed(HashMap::new());

        embed.title("The Queue");

        let mut i = 1;
        let mut song_list = vec![];
        for song in handler.queue().current_queue() {
            song_list.push((i, song.metadata().clone().title.unwrap()));
            i += 1;
        }

        let mut embed_desc: String = "".to_string();
        for x in song_list {
            embed_desc += &format!("{} - {}\n", x.0, x.1);
        }

        embed.description(embed_desc);

        ctx.channel_id()
            .send_message(&ctx.discord().http, |m| m.set_embed(embed))
            .await?;
    }

    Ok(())
}
