use crate::types::courses_as_hashmap;
use crate::types::day::DailyMenu;
use serde::Deserialize;
use std::collections::HashMap;

use super::common::{Course, Meta};

#[derive(Debug, Deserialize)]
pub struct WeeklyMenu {
    pub meta: Meta,
    pub timeperiod: String,
    pub mealdates: Vec<Day>,
}

impl From<WeeklyMenu> for Vec<(String, DailyMenu)> {
    fn from(val: WeeklyMenu) -> Self {
        let meta = val.meta;

        val.mealdates
            .into_iter()
            .map(|d| {
                (
                    d.date,
                    DailyMenu {
                        meta: meta.clone(),
                        courses: d.courses,
                    },
                )
            })
            .collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct Day {
    pub date: String,
    #[serde(deserialize_with = "courses_as_hashmap", default)]
    pub courses: HashMap<String, Course>,
}
