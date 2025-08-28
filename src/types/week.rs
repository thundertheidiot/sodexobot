use serde::Deserialize;
use std::collections::HashMap;

use super::common::{Course, Meta};

#[derive(Debug, Deserialize)]
pub struct WeeklyMenu {
    pub meta: Meta,
    pub timeperiod: String,
    pub mealdates: Vec<Day>,
}

#[derive(Debug, Deserialize)]
pub struct Day {
    pub date: String,
    pub courses: HashMap<String, Course>,
}
