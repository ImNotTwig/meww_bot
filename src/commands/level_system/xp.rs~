use crate::file_loader::open_level_enabler;
use crate::file_loader::open_server_levels;
use crate::{Context, Error};
use poise::serenity_prelude as sr;
use poise::serenity_prelude::CreateEmbed;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerMember {
    pub level: i64,
    pub total_xp: i64,
    pub current_xp: i64,
    pub xp_needed: i64,
    pub can_gain_xp: bool,
}

impl ServerMember {
    pub fn new() -> ServerMember {
        ServerMember {
            level: 0,
            total_xp: 0,
            current_xp: 0,
            xp_needed: 100,
            can_gain_xp: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerLevels {
    pub members: BTreeMap<String, ServerMember>,
}

impl ServerLevels {
    pub fn new() -> ServerLevels {
        ServerLevels {
            members: BTreeMap::new(),
        }
    }
}

#[poise::command(prefix_command, aliases("xp"))]
pub async fn check_xp(
    ctx: Context<'_>,
    #[description = "Which user's xp stats do you want to check. (Leave blank if you want to check yourself.)"]
    mut user: Option<sr::User>,
) -> Result<(), Error> {
    if user == None {
        user = Some(ctx.clone().author().to_owned());
    }

    let user_name = user.clone().unwrap().name.clone();
    let user_id = user.clone().unwrap().id.to_string().to_owned();
    let server_id = ctx.clone().guild_id().unwrap().to_string().to_owned();
    let mut levels = open_server_levels();
    let mut global_levels = levels.get("global").unwrap().clone();

    // checking if the server is in the levels dict
    if !levels.contains_key(&server_id) {
        let new_server = ServerLevels {
            members: BTreeMap::new(),
        };
        levels.insert(server_id.clone(), new_server);

        if !global_levels.clone().members.contains_key(&user_id) {
            global_levels
                .members
                .insert(user_id.clone(), ServerMember::new());
            levels.insert("global".to_string(), global_levels.clone());
        }

        let buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
        levels.serialize(&mut ser).unwrap();

        let mut level_file = fs::File::create("./src/commands/level_system/levels.json").unwrap();
        write!(
            level_file,
            "{}",
            String::from_utf8(ser.into_inner()).unwrap()
        )
        .unwrap();
        levels = open_server_levels();
    }
    // checking if the member is in the server dict
    if !levels
        .get(&server_id)
        .unwrap()
        .members
        .contains_key(&user_id)
    {
        let mut server_members = levels.get(&server_id).unwrap().members.clone();
        server_members.insert(user_id.clone(), ServerMember::new());
        let mut server_levels = levels.get(&server_id).unwrap().clone();
        server_levels.members = server_members;

        levels.insert(server_id.clone(), server_levels);

        if !global_levels.clone().members.contains_key(&user_id) {
            global_levels
                .members
                .insert(user_id.clone(), ServerMember::new());
            levels.insert("global".to_string(), global_levels.clone());
        }

        let buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
        levels.serialize(&mut ser).unwrap();

        let mut level_file = fs::File::create("./src/commands/level_system/levels.json").unwrap();
        write!(
            level_file,
            "{}",
            String::from_utf8(ser.into_inner()).unwrap()
        )
        .unwrap();
        levels = open_server_levels();
    }

    let user_pfp = user.unwrap().avatar_url().unwrap();

    let global_xp = levels
        .get("global")
        .unwrap()
        .members
        .get(&user_id)
        .unwrap()
        .total_xp;

    let global_level = levels
        .get("global")
        .unwrap()
        .members
        .get(&user_id)
        .unwrap()
        .level;

    let xp = levels
        .get(&server_id)
        .unwrap()
        .members
        .get(&user_id)
        .unwrap()
        .current_xp;

    let level = levels
        .get(&server_id)
        .unwrap()
        .members
        .get(&user_id)
        .unwrap()
        .level;

    let xp_needed = levels
        .get(&server_id)
        .unwrap()
        .members
        .get(&user_id)
        .unwrap()
        .xp_needed;

    let total_server_xp = levels
        .get(&server_id)
        .unwrap()
        .members
        .get(&user_id)
        .unwrap()
        .total_xp;

    let xp_needed_to_level_up = xp_needed - xp;
    let amount_per_box = xp_needed / 20;
    let current_boxes = xp / amount_per_box;
    let boxes_left = xp_needed_to_level_up / amount_per_box;

    let blue_squares = ":blue_square:"
        .to_string()
        .repeat(usize::try_from(current_boxes).unwrap());
    let white_squares = ":white_large_square:"
        .to_string()
        .repeat(usize::try_from(boxes_left).unwrap());

    ctx.channel_id()
        .send_message(&ctx.discord().http, |m| {
            m.embed(|e| {
                e.thumbnail(user_pfp)
                    .title(format!("{}'s Level Stats", user_name))
                    .field("Global xp", global_xp, true)
                    .field("Global Level", global_level, true)
                    .field(format!("{}'s Current xp", user_name), xp, true)
                    .field(
                        format!("{}'s Total xp in this Server", user_name),
                        total_server_xp,
                        true,
                    )
                    .field(format!("{}'s Level", user_name), level, true)
                    .field(
                        format!("Progress to leveling up in {}", ctx.guild().unwrap().name),
                        format!("{}{}", blue_squares, white_squares),
                        false,
                    )
            })
        })
        .await?;

    Ok(())
}

#[poise::command(prefix_command, aliases("levelswitch"))]
pub async fn levels(
    ctx: Context<'_>,
    #[description = "Do you want to turn the level system on or off?"] mut on_or_off: String,
) -> Result<(), Error> {
    let mut server_enabler = open_level_enabler();
    let server_id = ctx.clone().guild_id().unwrap().to_string();

    if &on_or_off.to_lowercase() == "on" || &on_or_off.to_lowercase() == "true" {
        server_enabler.insert(server_id, true);
        on_or_off = "on".to_string();
    } else if &on_or_off.to_lowercase() == "off" || &on_or_off == "false" {
        server_enabler.insert(server_id, false);
        on_or_off = "off".to_string();
    }

    let buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
    server_enabler.serialize(&mut ser).unwrap();

    let mut server_enabler_file =
        fs::File::create("./src/commands/level_system/server_enabler.json").unwrap();

    write!(
        server_enabler_file,
        "{}",
        String::from_utf8(ser.into_inner()).unwrap()
    )
    .unwrap();

    ctx.say(format!(
        "The level system for {} has been turned {}",
        ctx.guild().unwrap().name,
        on_or_off
    ))
    .await?;

    Ok(())
}

#[poise::command(prefix_command, aliases("lb"))]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let server_id = ctx.guild_id().unwrap().clone().to_string();
    let server_dict = open_server_levels().get(&server_id).unwrap().clone();
    let mut levels_dict_resorted = BTreeMap::new();

    for member in server_dict.members {
        let level = member.1.level;
        let total_xp = member.1.total_xp;
        let user = sr::UserId {
            0: member.0.parse::<u64>().unwrap(),
        }
        .to_user(&ctx.discord().http)
        .await
        .unwrap();

        let mut user_name = user
            .nick_in(&ctx.discord().http, server_id.parse::<u64>().unwrap())
            .await
            .clone();

        if user_name == None {
            user_name = Some(user.name.clone());
        }

        let user_name = user_name.unwrap();

        levels_dict_resorted.insert(total_xp, (user_name, level.to_string()));
    }

    let mut levels_vec = Vec::from_iter(levels_dict_resorted);

    levels_vec.sort_by(|a, b| {
        if a.0 < b.0 {
            Ordering::Greater
        } else if a.0 == b.0 {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    });

    while levels_vec.len() > 10 {
        levels_vec.pop();
    }

    let mut embed = CreateEmbed(HashMap::new());

    embed.title(format!(
        "{}'s leaderboard",
        ctx.clone().guild().unwrap().name
    ));

    for i in levels_vec {
        embed.field(
            i.1 .0,
            format!("Level: {}\nTotal Xp: {}", i.1 .1, i.0),
            false,
        );
    }

    ctx.channel_id()
        .send_message(&ctx.discord().http, |m| m.set_embed(embed))
        .await?;

    Ok(())
}

#[poise::command(prefix_command, aliases("givexp"))]
pub async fn give_xp(
    ctx: Context<'_>,
    #[description = "Which user do you want to give xp. (Leave blank if you want to give yourself xp.)"]
    mut user: Option<sr::User>,
    #[description = "How much xp do you want to give?"] amount: i64,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let mut levels_dict = open_server_levels();
    let mut global_dict = levels_dict.get("global").unwrap().clone();
    let mut server_dict = levels_dict.get(&guild_id).unwrap().clone();

    if user == None {
        user = Some(ctx.author().clone());
    }
    let user = user.unwrap();
    let user_id = user.id.to_string();

    let mut user_dict = server_dict.members.get(&user_id).unwrap().clone();
    let mut user_global_dict = global_dict.members.get(&user_id).unwrap().clone();

    user_dict.current_xp += amount;
    user_dict.total_xp += amount;
    user_global_dict.total_xp += amount;

    while user_dict.current_xp >= user_dict.xp_needed {
        if user_dict.current_xp > user_dict.xp_needed {
            user_dict.current_xp = user_dict.current_xp - user_dict.xp_needed;
        } else {
            user_dict.current_xp = 0;
        }

        user_dict.level += 1;
        user_global_dict.level += 1;
        user_dict.xp_needed = 5 * (user_dict.level.pow(2)) + (50 * user_dict.level) + 100;
    }

    global_dict
        .members
        .insert(user_id.clone(), user_global_dict);

    server_dict.members.insert(user_id.clone(), user_dict);

    levels_dict.insert("global".to_string(), global_dict);
    levels_dict.insert(guild_id.clone(), server_dict);

    let buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
    levels_dict.serialize(&mut ser).unwrap();

    let mut levels_file = fs::File::create("./src/commands/level_system/levels.json").unwrap();

    write!(
        levels_file,
        "{}",
        String::from_utf8(ser.into_inner()).unwrap()
    )
    .unwrap();

    ctx.say(format!(
        "{} has given {} {} xp",
        ctx.author_member().await.unwrap().display_name(),
        user.name,
        amount,
    ))
    .await?;

    Ok(())
}

#[poise::command(prefix_command, aliases("removexp"))]
pub async fn remove_xp(
    ctx: Context<'_>,
    #[description = "Which user do you want to remove xp from. (Leave blank if you want to remove xp from yourself.)"]
    mut user: Option<sr::User>,
    #[description = "How much xp do you want to remove?"] amount: i64,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let mut levels_dict = open_server_levels();
    let mut global_dict = levels_dict.get("global").unwrap().clone();
    let mut server_dict = levels_dict.get(&guild_id).unwrap().clone();

    if user == None {
        user = Some(ctx.author().clone());
    }
    let user = user.unwrap();
    let user_id = user.id.to_string();

    let mut user_dict = server_dict.members.get(&user_id).unwrap().clone();
    let mut user_global_dict = global_dict.members.get(&user_id).unwrap().clone();

    user_dict.current_xp -= amount;
    user_dict.total_xp -= amount;
    user_global_dict.total_xp -= amount;

    while user_dict.current_xp.is_negative() {
        user_dict.level -= 1;
        user_global_dict.level -= 1;
        user_dict.xp_needed = 5 * (user_dict.level.pow(2)) + (50 * user_dict.level) + 100;
        user_dict.current_xp += user_dict.xp_needed;
    }
    if user_dict.total_xp < 0 {
        user_dict.total_xp = 0;
    }
    if user_global_dict.total_xp < 0 {
        user_global_dict.total_xp = 0;
    }

    global_dict
        .members
        .insert(user_id.clone(), user_global_dict);

    server_dict.members.insert(user_id.clone(), user_dict);

    levels_dict.insert("global".to_string(), global_dict);
    levels_dict.insert(guild_id.clone(), server_dict);

    let buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
    levels_dict.serialize(&mut ser).unwrap();

    let mut levels_file = fs::File::create("./src/commands/level_system/levels.json").unwrap();

    write!(
        levels_file,
        "{}",
        String::from_utf8(ser.into_inner()).unwrap()
    )
    .unwrap();

    ctx.say(format!(
        "{} has removed {} xp from {}",
        ctx.author_member().await.unwrap().display_name(),
        amount,
        user.name,
    ))
    .await?;

    Ok(())
}
