use crate::file_loader::open_server_levels;
use crate::{Context, Error};
use poise::serenity_prelude::{self as sr, CreateEmbed, Embed};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerMember {
    pub level: u64,
    pub total_xp: u64,
    pub current_xp: u64,
    pub xp_needed: u64,
    pub can_gain_xp: bool,
}

impl ServerMember {
    fn new() -> ServerMember {
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

    // checking if the server is in the levels dict
    if !levels.contains_key(&server_id) {
        let new_server = ServerLevels {
            members: BTreeMap::new(),
        };
        levels.insert(server_id.clone(), new_server);

        let buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
        levels.serialize(&mut ser).unwrap();

        let mut levels = fs::File::create("./src/commands/level_system/levels.json").unwrap();
        write!(levels, "{}", String::from_utf8(ser.into_inner()).unwrap()).unwrap();
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

        let buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
        levels.serialize(&mut ser).unwrap();

        let mut levels = fs::File::create("./src/commands/level_system/levels.json").unwrap();
        write!(levels, "{}", String::from_utf8(ser.into_inner()).unwrap()).unwrap();
    }

    let user_pfp = ctx.author().clone().avatar_url().unwrap();

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
                    .field("Global Xp", global_xp, true)
                    .field("Global Level", global_level, true)
                    .field(format!("{}'s Xp", user_name), xp, true)
                    .field(format!("{}'s Level", user_name), level, true)
                    .field(
                        format!("Progress to leveling up in {}", user_name),
                        format!("{}{}", blue_squares, white_squares),
                        false,
                    )
            })
        })
        .await?;

    Ok(())
}
