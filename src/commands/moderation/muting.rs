use crate::{read_config, Context, Error};
use poise::serenity_prelude as sr;
use poise::serenity_prelude::CacheHttp;
use sr::RoleId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use chrono::{Utc, Duration};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnmutedTime { pub unmuted_time: BTreeMap<String, String> }

// used for removing the last letter in a time arguement to get the time units
pub fn rem_last_char(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next_back();
     chars.as_str()
}

// ---------MUTE COMMAND---------------------------------------------------------------------------------------

#[poise::command(prefix_command)]
pub async fn mute(
    ctx: Context<'_>,
    #[description = "User to mute."]
    mut member: sr::Member,
    #[description = "How long to mute them for, eg: '1m' (1 minute) or '3h' (3 hours), if no time is supplied the time will be infinite."]
    mut time: Option<String>,
    #[description = "Reason you are muting them for. (You can leave this blank.)"]
    mut reason: Option<String>
) -> Result<(), Error> {
    // initializing variables
    let mut time_units = 0;
    let mut timeframe = "seconds";
    let mut time_muted = Utc::now();
    let mut time_muted_as_string = String::new();
    
    // opening the mute roles dict
    let file = fs::File::open("./src/commands/moderation/mute_roles.json").unwrap();
    let json_value: serde_json::Value = serde_json::from_reader(file).unwrap();
    let json = json_value.to_string();
    let mute_roles = serde_json::from_str::<BTreeMap<String, String>>(&json)?;
    
    let guild_id = ctx.guild_id().unwrap().to_string();

    // checking if the server has a mute role associated with it
    if mute_roles.contains_key(&guild_id) == false {
        let config = read_config().unwrap();
        ctx.say(format!("{} does not have a mute role, please set one uaing {}muterole", &ctx.guild().unwrap().name, config.prefix)).await?;
    } 
    
    // checking if the user is already muted
    if member.roles.contains(&RoleId::from(mute_roles.get(&guild_id).unwrap().parse::<u64>().unwrap())) {
        ctx.say("User is already muted.").await?;
        return Ok(())
    }

    // if the time argument is None
    if &time != &None {
        // if the time argument started with a number
        if time.as_ref().unwrap().chars().nth(0).unwrap().is_numeric(){
            // if the time argument ends with one of these letters assign a timeframe
            match time.as_ref().unwrap().chars().last() {
                Some('s') => timeframe = "seconds",
                Some('m') => timeframe = "minutes",
                Some('h') => timeframe = "hours",
                Some('d') => timeframe = "days",
                Some('M') => timeframe = "months",
                Some('y') => timeframe = "years",
                Some('D') => timeframe = "decades",
                Some('c') => timeframe = "centuries",
                // else assume a reason was supplied but no time
                _ => {
                    println!("incorrect character supplied. Assuming no time was given and setting the argument to reason only.");
                    reason = time.clone();
                    time = None;
                },
            }
        // if the first character is not a number assume a reason was supplied without a time
        } else {
            reason = time.clone();
            time = None;
        }
    }
    
    // if time is not None figure out how long the user should be muted
    if &time == &None {
        time_units = 0;
        timeframe = "seconds";
    } else {
        // TODO: handle this to make sure the number the user gave wasnt too big
        time_units = rem_last_char(time.as_ref().unwrap()).parse().unwrap();
        match timeframe {
            "seconds" => {time_muted = Utc::now() + Duration::seconds(time_units)},
            "minutes" => {time_muted = Utc::now() + Duration::minutes(time_units)},
            "hours" => {time_muted = Utc::now() + Duration::hours(time_units)},
            "days" => {time_muted = Utc::now() + Duration::days(time_units)},
            "weeks" => {time_muted = Utc::now() + Duration::weeks(time_units)},
            "months" => {time_muted = Utc::now() + Duration::weeks(time_units * 4)},
            "years" => {time_muted = Utc::now() + Duration::weeks(time_units * 4 * 12)},
            "decades" => {time_muted = Utc::now() + Duration::weeks(time_units * 4 * 12 * 10)},
            "centuries" => {time_muted = Utc::now() + Duration::weeks(time_units * 4 * 12 * 100)},
            _ => {},
        }
        // converting the time the user is supposed to be muted to a string
        time_muted_as_string = format!("{}", time_muted);
    
        // opening the unmuted time dict 
        let unmute_times = fs::File::open("./src/commands/moderation/unmuted_times.json").unwrap();
        let unmute_value: serde_json::Value = serde_json::from_reader(&unmute_times).unwrap();
        let unmute_json = unmute_value.to_string();
        let mut unmute_times_dict = serde_json::from_str::<BTreeMap<String, UnmutedTime>>(&unmute_json)?;
        
        // if the unmute_times_dict doesnt have the guild inserted yet
        if unmute_times_dict.contains_key(&guild_id) == false {
            unmute_times_dict.insert(guild_id.clone(), UnmutedTime { unmuted_time: BTreeMap::new() });
        }
        
        // setting the user id 
        let user_id = member.user.id.to_string();
        
        // getting the guild's dict within the entire unmute_times_dict
        let mut users_muted = unmute_times_dict.clone().get(&guild_id).unwrap().clone();
        // inserting the user with the time they should be unmuted into the guild's dict
        users_muted.unmuted_time.insert(user_id, time_muted_as_string);
        // inserting the guild back into the unmute_times_dict
        unmute_times_dict.insert(guild_id.clone(), users_muted.clone()); 
        
        // writing the unmute_times_dict to file
        let buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
        unmute_times_dict.serialize(&mut ser).unwrap();
        
        let mut unmute_times = fs::File::create("./src/commands/moderation/unmuted_times.json").unwrap();
        write!(unmute_times, "{}", String::from_utf8(ser.into_inner()).unwrap()).unwrap();
    }
        // getting the role id and adding the muterole to user    
        let roleid = mute_roles.get(&guild_id).unwrap().parse::<u64>().unwrap();
        let http = ctx.discord().http();
        member.add_role(http, roleid).await?;
        
        // checking the reason and time args to see which message to send 
        if reason == None {
            if time_units != 0 {
                if time_units == 1 {
                    timeframe = timeframe.strip_suffix("s").unwrap();
                    if timeframe == "centurie" {
                        timeframe = "century";
                    }
                }
                ctx.say(format!("{} has been muted for {} {}.", &member.user.name, &time_units, &timeframe)).await?;
            } else {
                ctx.say(format!("{} has been muted.", &member.user.name)).await?;
            }
        } else if reason != None {
            if time_units != 0 {
                if time_units == 1 {
                    timeframe = timeframe.strip_suffix("s").unwrap();
                    if timeframe == "centurie" {
                        timeframe = "century";
                    }
                }
                ctx.say(format!("{} has been muted for {} for {} {}.", &member.user.name, &reason.unwrap(), &time_units, &timeframe)).await?;
            } else {
                ctx.say(format!("{} has been muted for {}.", &member.user.name, &reason.unwrap())).await?;
            }
        }

    Ok(())
}

// ---------UNMUTE COMMAND-------------------------------------------------------------------------------------

#[poise::command(prefix_command)]
pub async fn unmute(
    ctx: Context<'_>,
    #[description = "User to unmute."]
    mut member: sr::Member,
) -> Result<(), Error> {
    // getting user and guild ids to make things easier
    let user_id = member.user.id.to_string();
    let guild_id = ctx.clone().guild_id().unwrap().to_string();
    
    // opening the muteroles dict file
    let file = fs::File::open("./src/commands/moderation/mute_roles.json").unwrap();
    let json: serde_json::Value = serde_json::from_reader(file).unwrap();
    let json = json.to_string();
    let mute_roles = serde_json::from_str::<BTreeMap<String, String>>(&json)?;
    
    // if there is no muterole associated with this server
    if mute_roles.clone().contains_key(&guild_id) == false {
        let config = read_config().unwrap();
        ctx.say(format!("{} does not have a mute role, please set one using {}muterole", &ctx.guild().unwrap().name, config.prefix)).await?;
    }
    
    // opening the unmuted times dict 
    let unmute_times = fs::File::open("./src/commands/moderation/unmuted_times.json").unwrap();
    let unmute_value: serde_json::Value = serde_json::from_reader(&unmute_times).unwrap();
    let unmute_json = unmute_value.to_string();
    
    //get the dict thats <GuildId <UserId, WhenUserIsUnmuted>>
    let mut unmute_times_dict = serde_json::from_str::<BTreeMap<String, UnmutedTime>>(&unmute_json).unwrap();
    
    // get the user dict that we want from the unmute times dict 
    let mut user_dict = unmute_times_dict.get(&*guild_id).unwrap().clone();
    
    // remove the user from the dict
    user_dict.unmuted_time.remove(&user_id);
    
    // recombine the dicts
    unmute_times_dict.insert(guild_id.clone(), user_dict.clone());
    
    // write to file
    let buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
    unmute_times_dict.serialize(&mut ser).unwrap();
    let mut unmute_times = fs::File::create("./src/commands/moderation/unmuted_times.json").unwrap();
    write!(unmute_times, "{}", String::from_utf8(ser.into_inner()).unwrap()).unwrap();
    
    // removing the role from user
    let roleid = mute_roles.get(&guild_id).unwrap().parse::<u64>().unwrap();
    let http = ctx.discord().http();
    member.remove_role(http, roleid).await?;
    ctx.say(format!("{} has been un-muted", member.user.name)).await?;

    Ok(())
}

// ---------MUTEROLE COMMAND-----------------------------------------------------------------------------------

#[poise::command(prefix_command)]
pub async fn muterole(
    ctx: Context<'_>,
    #[description = "Role to set."]
    role: sr::Role,
) -> Result<(), Error> {
    // opening the muteroles dict
    let file = fs::File::open("./src/commands/moderation/mute_roles.json").unwrap();
    let json: serde_json::Value = serde_json::from_reader(file).unwrap();
    let json = json.to_string();
    let mut mute_roles = serde_json::from_str::<HashMap<String, String>>(&json)?;
    
    // inserting or replacing the entry for the guild with the role id 
    mute_roles.insert(ctx.guild_id().unwrap().to_string(), role.id.to_string());
    
    let buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
    mute_roles.serialize(&mut ser).unwrap();
    let mut mute_roles_file = fs::File::create("./src/commands/moderation/mute_roles.json").unwrap();
    write!(mute_roles_file, "{}", String::from_utf8(ser.into_inner()).unwrap()).unwrap();
    
    // tell them we set the role
    ctx.say(format!("{} has been set as the mute role for {}", role.name, ctx.guild().unwrap().name)).await?;
    
    Ok(())
}
