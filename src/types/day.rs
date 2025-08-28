use serde::Deserialize;
use std::collections::HashMap;

use super::common::{Course, Meta};

#[derive(Debug, Deserialize)]
pub struct DailyMenu {
    pub meta: Meta,
    pub courses: HashMap<String, Course>,
}
