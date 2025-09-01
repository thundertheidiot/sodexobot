use either::Either;
use either::Either::Left;
use either::Either::Right;
use serde::Deserialize;
use serde::Deserializer;
use std::collections::HashMap;

pub fn deserialize_as_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let map: HashMap<String, T> = HashMap::deserialize(deserializer)?;
    Ok(map.into_values().collect())
}

#[derive(Debug, Clone, Deserialize)]
pub struct Meta {
    pub generated_timestamp: i64,
    pub ref_url: String,
    pub ref_title: String,
    pub restaurant_mashie_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Course {
    pub title_fi: Option<String>,
    pub title_en: Option<String>,
    pub category: Option<String>,
    pub meal_category: Option<String>,
    #[serde(rename(deserialize = "dietcodes"))]
    #[serde(deserialize_with = "deserialize_diet_info")]
    #[serde(default)]
    pub diet_info: DietInfo,
    // Repeat of dietcodes
    pub properties: Option<String>,
    #[serde(rename(deserialize = "additionalDietInfo"))]
    #[serde(default)]
    pub additional_diet_info: AdditionalDietInfo,
    pub price: Option<String>,

    pub recipes: Option<RecipesWrapper>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdditionalDietInfo {
    #[serde(rename(deserialize = "dietcodeImages"))]
    #[serde(deserialize_with = "deserialize_food_info")]
    #[serde(default)]
    pub food_info: FoodInfo,
    pub allergens: Option<String>,
}

impl Default for AdditionalDietInfo {
    fn default() -> Self {
        AdditionalDietInfo {
            food_info: FoodInfo::default(),
            allergens: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FoodInfo {
    pub co2: bool,
    pub heart: bool,
    pub vegan: bool,
    pub student_recommendation: bool,
    pub pork: bool,
    pub fi_meat: bool,
    pub eu_meat: bool,
    pub other_meat: bool,
}

fn deserialize_food_info<'de, D>(deserializer: D) -> Result<FoodInfo, D::Error>
where
    D: Deserializer<'de>,
{
    let mut food_info = FoodInfo::default();

    let urls: Option<Vec<String>> = Option::deserialize(deserializer)?;

    match urls {
        Some(urls) => {
            if urls.contains(
                &"https://www.sodexo.fi/sites/default/themes/sodexo/images/sodexo-leaf.svg"
                    .to_string(),
            ) {
                food_info.co2 = true;
            }

            if urls.contains(
                &"https://www.sodexo.fi/sites/default/themes/sodexo/images/sydan.svg".to_string(),
            ) {
                food_info.heart = true;
            }

            if urls.contains(
                &"https://www.sodexo.fi/sites/default/themes/sodexo/images/vege.svg".to_string(),
            ) {
                food_info.vegan = true;
            }

            if urls.contains(
                &"https://www.sodexo.fi/sites/default/themes/sodexo/images/omena.svg".to_string(),
            ) {
                food_info.student_recommendation = true;
            }

            if urls.contains(
                &"https://www.sodexo.fi/sites/default/themes/sodexo/images/possu.svg".to_string(),
            ) {
                food_info.pork = true;
            }

            if urls.contains(
                &"https://www.sodexo.fi/sites/default/themes/sodexo/images/liha-fi-new.svg"
                    .to_string(),
            ) {
                food_info.fi_meat = true;
            }

            if urls.contains(
                &"https://www.sodexo.fi/sites/default/themes/sodexo/images/liha-eu-new.svg"
                    .to_string(),
            ) {
                food_info.eu_meat = true;
            }

            if urls.contains(
                &"https://www.sodexo.fi/sites/default/themes/sodexo/images/liha-muu-new.svg"
                    .to_string(),
            ) {
                food_info.other_meat = true;
            }

            Ok(food_info)
        }
        None => Ok(food_info),
    }
}

#[derive(Debug, Clone, Default)]
pub struct DietInfo {
    pub gluten_free: bool,
    pub lactose_free: bool,
    pub milk_free: bool,
    pub low_lactose: bool,
}

fn deserialize_diet_info<'de, D>(deserializer: D) -> Result<DietInfo, D::Error>
where
    D: Deserializer<'de>,
{
    let mut diet_info = DietInfo::default();

    let codes: Option<String> = Option::deserialize(deserializer)?;

    match codes {
        Some(codes) => {
            for i in codes.split(',').collect::<Vec<&str>>() {
                match i {
                    "G" => diet_info.gluten_free = true,
                    "L" => diet_info.lactose_free = true,
                    "M" => diet_info.milk_free = true,
                    "VL" => diet_info.low_lactose = true,
                    _ => (),
                }
            }

            Ok(diet_info)
        }
        None => Ok(diet_info),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecipesWrapper {
    #[serde(flatten)]
    #[serde[deserialize_with = "deserialize_as_vec"]]
    pub recipes: Vec<Recipe>,
    #[serde(rename(deserialize = "hideAll"))]
    pub hide_all: Option<HideAll>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HideAll {
    #[serde(rename(deserialize = "dietcodes"))]
    #[serde[deserialize_with = "deserialize_diet_info"]]
    #[serde(default)]
    pub diet_info: DietInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub ingredients: StringOrEmptyList,
    pub nutrients: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct StringOrEmptyList {
    #[serde(with = "either::serde_untagged")]
    inner: Either<String, Vec<()>>,
}

impl ToString for StringOrEmptyList {
    fn to_string(&self) -> String {
        match &self.inner {
            Left(s) => s.clone(),
            Right(_) => "N/A".to_string(),
        }
    }
}
