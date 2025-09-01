use crate::types::common::Course;
use serde::Deserialize;
use serde::Deserializer;
use std::collections::HashMap;

pub mod common;
pub mod day;
// pub mod diet_info;
pub mod week;

fn courses_as_hashmap<'de, D>(deserializer: D) -> Result<HashMap<String, Course>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Helper {
        Map(HashMap<String, Course>),
        List(Vec<Course>),
    }

    match Helper::deserialize(deserializer)? {
        Helper::Map(map) => Ok(map),
        Helper::List(_) => Ok(HashMap::new()), // discard list, return empty
    }
}

#[test]
fn test_deserialize_daily_menu() {
    let daily = std::fs::read_to_string("tests/daily.json").expect("no file");

    let menu: DailyMenu = serde_json::from_str(&daily).expect("unable to parse json");

    println!("{:#?}", menu);
}

#[test]
fn test_deserialize_weekly_menu() {
    let weekly = std::fs::read_to_string("tests/weekly.json").expect("no file");

    let menu: WeeklyMenu = serde_json::from_str(&weekly).expect("unable to parse json");

    println!("{:#?}", menu);
}
