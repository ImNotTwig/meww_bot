use crate::{SpamCount, UnmutedTime};
use serde_json;
use std::collections::BTreeMap;
use std::fs;

use crate::commands::level_system::xp::ServerLevels;

// server levels opener
pub fn open_server_levels() -> BTreeMap<String, ServerLevels> {
    let levels_file = fs::File::open("./src/commands/level_system/levels.json").unwrap();
    let levels_value: serde_json::Value = serde_json::from_reader(&levels_file).unwrap();
    let levels_json = levels_value.to_string();

    serde_json::from_str::<BTreeMap<String, ServerLevels>>(&levels_json).unwrap()
}

// unmute times opener
pub fn open_unmute_times() -> BTreeMap<String, UnmutedTime> {
    let mut unmute_times =
        fs::read_to_string("./src/commands/moderation/unmuted_times.json").unwrap();

    if unmute_times == "" {
        unmute_times = "{}".to_string();
    }

    let unmute_value: serde_json::Value = serde_json::from_str(&unmute_times).unwrap();
    let unmute_json = unmute_value.to_string();

    serde_json::from_str::<BTreeMap<String, UnmutedTime>>(&unmute_json).unwrap()
}

// mute roles opener
pub fn open_mute_roles() -> BTreeMap<String, String> {
    let mut mute_roles_file =
        fs::read_to_string("./src/commands/moderation/mute_roles.json").unwrap();

    if mute_roles_file == "" {
        mute_roles_file = "{}".to_string();
    }

    let mute_roles_json: serde_json::Value = serde_json::from_str(&mute_roles_file).unwrap();

    let mute_roles_json = mute_roles_json.to_string();

    serde_json::from_str::<BTreeMap<String, String>>(&mute_roles_json).unwrap()
}

// spam count opener
pub fn open_spam_count() -> BTreeMap<String, BTreeMap<String, SpamCount>> {
    let file = fs::File::open("./src/commands/moderation/spam_count.json").unwrap();
    let json_value: serde_json::Value = serde_json::from_reader(file).unwrap();
    let json = json_value.to_string();

    serde_json::from_str::<BTreeMap<String, BTreeMap<String, SpamCount>>>(&json).unwrap()
}
