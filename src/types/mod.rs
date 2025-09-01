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
    #[allow(dead_code)] // the Vec<()> is essential for parsing an empty list correctly
    enum Helper {
        Map(HashMap<String, Course>),
        List(Vec<()>),
    }

    match Helper::deserialize(deserializer)? {
        Helper::Map(map) => Ok(map),
        Helper::List(_) => Ok(HashMap::new()), // discard list, return empty
    }
}

#[test]
fn test_deserialize_daily_menu() {
    use crate::types::day::DailyMenu;

    let daily = std::fs::read_to_string("tests/daily.json").expect("no file");

    let menu: DailyMenu = serde_json::from_str(&daily).expect("unable to parse json");

    println!("{:#?}", menu);
}

#[test]
fn additional_diet_info() {
    use crate::types::common::AdditionalDietInfo;

    let json = r#"{
				"dietcodeImages": [
					"https://www.sodexo.fi/sites/default/themes/sodexo/images/sodexo-leaf.svg",
					"https://www.sodexo.fi/sites/default/themes/sodexo/images/sydan.svg",
					"https://www.sodexo.fi/sites/default/themes/sodexo/images/vege.svg",
					"https://www.sodexo.fi/sites/default/themes/sodexo/images/omena.svg"
				],
				"allergens": "Chili, Herneet, Hiivauute, Kaura, Korianteri, Rikkidioksidi ja sulfiitit, Sipuli, Sitrukset, Soijapavut, Valkosipuli"
}"#;

    let json2 = r#"{
				"dietcodeImages": [
				],
				"allergens": "Chili, Herneet, Hiivauute, Kaura, Korianteri, Rikkidioksidi ja sulfiitit, Sipuli, Sitrukset, Soijapavut, Valkosipuli"
}"#;

    let json3 = r#"{
				"allergens": "Chili, Herneet, Hiivauute, Kaura, Korianteri, Rikkidioksidi ja sulfiitit, Sipuli, Sitrukset, Soijapavut, Valkosipuli"
}"#;

    let _: AdditionalDietInfo = serde_json::from_str(json).expect("unable to parse json");
    let _: AdditionalDietInfo = serde_json::from_str(json2).expect("unable to parse json");
    let _: AdditionalDietInfo = serde_json::from_str(json3).expect("unable to parse json");
}

#[test]
fn deserialize_empty_day() {
    use crate::types::day::DailyMenu;

    let json = r#"{"meta":{"generated_timestamp":1756747242,"ref_url":"https:\/\/www.sodexo.fi\/ravintolat\/kokkola\/savonia-amk-centria-campus","ref_title":"Campusravintola","restaurant_mashie_id":"FI739646K"},"courses":[]}"#;

    let _: DailyMenu = serde_json::from_str(json).expect("unable to parse json");
}

#[test]
fn daily_menu_failure() {
    use crate::types::day::DailyMenu;
    let daily = std::fs::read_to_string("tests/2025-09-02.json").expect("no file");

    let menu: DailyMenu = serde_json::from_str(&daily).expect("unable to parse json");

    println!("{:#?}", menu);
}

#[test]
fn test_deserialize_weekly_menu() {
    use crate::types::week::WeeklyMenu;

    let weekly = std::fs::read_to_string("tests/weekly.json").expect("no file");

    let menu: WeeklyMenu = serde_json::from_str(&weekly).expect("unable to parse json");

    println!("{:#?}", menu);
}
