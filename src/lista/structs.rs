use either::Either;
use serde::Deserialize;
use serde::Deserializer;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct DailyMenu {
    pub meta: Meta,
    #[serde[deserialize_with = "deserialize_as_vec"]]
    pub courses: Vec<Course>,
}

#[derive(Debug, Deserialize)]
pub struct WeeklyMenu {
    pub meta: Meta,
    pub timeperiod: String,
    pub mealdates: Vec<Day>,
}

#[derive(Debug, Deserialize)]
pub struct Day {
    pub date: String,
    #[serde[deserialize_with = "deserialize_as_vec"]]
    pub courses: Vec<Course>,
}

fn deserialize_as_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let map: HashMap<String, T> = HashMap::deserialize(deserializer)?;
    Ok(map.into_values().collect())
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub generated_timestamp: i64,
    pub ref_url: String,
    pub ref_title: String,
    pub restaurant_mashie_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Course {
    pub title_fi: Option<String>,
    pub title_en: Option<String>,
    pub category: Option<String>,
    pub meal_category: Option<String>,
    pub dietcodes: Option<String>,
    pub properties: Option<String>,
    pub additionalDietInfo: AdditionalDietInfo,
    pub price: Option<String>,

    pub recipes: Option<RecipesWrapper>,
}

#[derive(Debug, Deserialize)]
pub struct AdditionalDietInfo {
    pub dietcodeImages: Option<Vec<String>>,
    pub allergens: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecipesWrapper {
    #[serde(flatten)]
    #[serde[deserialize_with = "deserialize_as_vec"]]
    pub recipes: Vec<Recipe>,
    pub hideAll: Option<HideAll>,
}

#[derive(Debug, Deserialize)]
pub struct HideAll {
    pub dietcodes: String,
}

#[derive(Debug, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub ingredients: StringOrEmptyList,
    pub nutrients: String,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct StringOrEmptyList {
    #[serde(with = "either::serde_untagged")]
    inner: Either<String, Vec<()>>,
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
