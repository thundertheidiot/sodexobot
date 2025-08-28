use crate::types::courses_as_hashmap;
use serde::Deserialize;
use std::collections::HashMap;

use super::common::{Course, Meta};

#[derive(Debug, Deserialize)]
pub struct DailyMenu {
    pub meta: Meta,
    #[serde(deserialize_with = "courses_as_hashmap", default)]
    pub courses: HashMap<String, Course>,
}
