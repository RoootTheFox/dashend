use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};

use regex::Regex;
use reqwest::Client;
use rocket_db_pools::Connection;

use crate::{structs::{DBUser, GDMessage, GenericError}, Db};

pub fn parse_gj_messages_response(meow: String) -> Result<Vec<GDMessage>, GenericError> {
    if meow == "-2" {
        // no messages :3
        return Ok(Vec::new());
    }

    meow.split('|')
        .map(|s| -> Result<GDMessage, GenericError> {
            let mut i = 0; // counter
            let mut last_key = 0;
            let mut values: HashMap<i32, &str> = HashMap::new();
            s.split(':').try_for_each(|a| -> Result<(), GenericError> {
                i += 1;
                if i % 2 == 1 {
                    // key
                    last_key = a.parse::<i32>()?;
                } else {
                    // val
                    values.insert(last_key, a.trim_end_matches(' '));
                }
                Ok(())
            })?;

            let mut found_null_byte = false;
            let subject_value: Vec<u8> = values
                .get(&4) // 4 = from account id
                .unwrap_or(&"")
                .trim_end_matches(' ')
                .as_bytes()
                .iter()
                .filter_map(|a| {
                    if found_null_byte || *a == 0 {
                        found_null_byte = true;
                        return None;
                    }
                    Some(*a)
                })
                .collect();
            let decoded = easy_base64::decode(subject_value.as_slice());

            Ok(GDMessage {
                id: values.get(&1).unwrap_or(&"0").to_string(), // 1 = message id
                from: values.get(&2).unwrap_or(&"0").parse().unwrap_or(-1), // 2 = account id
                subject: String::from_utf8_lossy(&decoded).to_string(),
            })
        })
        .collect()
}

pub async fn checkdiscordusername(mut conn: Connection<Db>, discord_snowflake: String, id: u32) {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    sqlx::query_as!(DBUser, "UPDATE users SET check_timeout = ? WHERE id = ?", time, id).execute(&mut **conn).await.unwrap();
    
    let client = Client::new();

    let regex = Regex::new(r".*:").unwrap();
    let snowflake = regex.replace_all(&discord_snowflake, "");

    let regex = Regex::new(r":|\d").unwrap();
    let username = regex.replace_all(&discord_snowflake, "");

    //dbg!(username);

    let req = client.get(format!("https://discord.com/api/v9/users/{}", snowflake))
    .header("Authorization", format!("Bot {}", dotenvy::var("DC_BOT_TOKEN").unwrap()))
    .send()
    .await
    .unwrap()
    .text()
    .await
    .unwrap();

    let crack = ajson::get(&req, "username").unwrap().unwrap();

    if crack.to_string() == username.to_string() {
        println!("dont change username")
    } else {
        println!("change username");
        sqlx::query!(
            r#"UPDATE profiles SET social_discord = ? WHERE id = ?"#,
            format!("{}:{}", crack.to_string(), snowflake.to_string().as_str()), id
        )
        .execute(&mut **conn)
        .await
        .unwrap();
    }
}