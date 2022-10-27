use crate::{Context, Error};
use poise::serenity_prelude as sr;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::ops::Deref;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerMember {
    pub level: u64,
    pub total_xp: u64,
    pub current_xp: u64,
    pub xp_needed: u64,
    pub can_gain_xp: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerLevels {
    pub members: BTreeMap<String, ServerMember>,
}

#[poise::command(prefix_command)]
pub async fn xp(
    ctx: Context<'_>,
    #[description = "Which user's xp stats do you want to check. (Leave blank if you want to check yourself.)"]
    mut user: Option<sr::User>,
) -> Result<(), Error> {
    let levels_file = fs::File::open("./src/commands/level_system/levels.json").unwrap();
    let levels_value: serde_json::Value = serde_json::from_reader(&levels_file).unwrap();
    let levels_json = levels_value.to_string();
    let mut levels_dict = serde_json::from_str::<BTreeMap<String, ServerLevels>>(&levels_json)?;

    if user == None {
        user = Some(ctx.clone().author().to_owned());
    }

    let user_id = user.unwrap().id;
    let server_id = ctx.clone().guild_id().unwrap();

    Ok(())
}
