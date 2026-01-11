use chrono::DateTime;
use chrono::Utc;
use std::collections::HashMap;
pub mod time_parser;

pub struct Assignment {
    pub id: u64, // doubles as message id
    pub title: String,
    pub deadline: DateTime<Utc>,
    pub assignment_type: ReminderType,
    pub completed_reminders: Vec<Reminder>,
    pub channel_id: u64,
    pub guild_id: u64,
    pub completed_by: Vec<u64>, // user ids who have reacted
}

impl Assignment {
    pub fn reminders<'a>(&self, config: &'a ReminderConfig) -> Option<Vec<&'a Reminder>> {
        let guild_id = self.guild_id;

        let reminder_types = config.get(&guild_id)?;
        let reminders = reminder_types.get(&self.assignment_type)?;

        Some(
            reminders
                .iter()
                .filter(|r| !self.completed_reminders.contains(*r))
                .collect::<Vec<&Reminder>>(),
        )
    }
}

type ReminderType = String;
type Reminder = String;

type Reminders = HashMap<ReminderType, Vec<Reminder>>;
type ReminderConfig = HashMap<u64, Reminders>; // guild id

pub struct AssignmentState {
    pub reminder_config: ReminderConfig,
    pub assignments: Vec<Assignment>,
}

pub mod tests {
    use std::time::Instant;

    use chrono::Local;

    use super::*;

    #[test]
    fn test_reminder() {
        let mut reminders: Reminders = HashMap::new();
        let r = vec!["1d".into(), "12h".into(), "1h".into()];
        reminders.insert("short".into(), r);

        let mut reminderconfig: ReminderConfig = HashMap::new();
        reminderconfig.insert(1, reminders);

        let assignment = Assignment {
            id: 1,
            title: "Test assignment".into(),
            deadline: Local::now().into(),
            assignment_type: "short".into(),
            completed_reminders: vec!["1d".into()],
            channel_id: 1,
            guild_id: 1,
            completed_by: Vec::new(),
        };

        let reminders = assignment.reminders(&reminderconfig).unwrap();

        assert_eq!(reminders, vec![&Into::<String>::into("12h"), &"1h".into()])
    }
}
