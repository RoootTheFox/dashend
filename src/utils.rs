use std::collections::HashMap;

use crate::structs::GDMessage;

pub fn parse_gj_messages_response(meow: String) -> Vec<GDMessage> {
    if meow == "-2" {
        // no messages :3
        return Vec::new();
    }

    meow.split('|')
        .map(|s| {
            let mut i = 0; // counter
            let mut last_key = 0;
            let mut values: HashMap<i32, &str> = HashMap::new();
            s.split(':').for_each(|a| {
                i += 1;
                if i % 2 == 1 {
                    // key
                    last_key = a.parse::<i32>().unwrap(); // (this should never fail so unwrap is fine)
                } else {
                    // val
                    values.insert(last_key, a.trim_end_matches(' '));
                }
            });

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

            GDMessage {
                id: values.get(&1).unwrap_or(&"0").to_string(), // 1 = message id
                from: values.get(&2).unwrap_or(&"0").parse().unwrap_or(-1), // 2 = account id
                subject: String::from_utf8_lossy(&decoded).to_string(),
            }
        })
        .collect()
}
