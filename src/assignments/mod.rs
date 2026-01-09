use chrono_tz::Europe::Helsinki;
mod parse_time;

struct Assignment {
    id: u64, // doubles as message id
    title: String,
    deadline: DateTime<Helsinki>,
    assignment_type: ReminderType,
    completed_reminders: Vec<Reminder>,
    channel_id: u64,
    guild_id: u64,
    completed_by: Vec<u64>, // user ids who have reacted
}

type ReminderType = String;
type Reminder = String;

// reminderconfig
// HashMap<guild id, HashMap<String, Vec<u64>>
// map of remidner types
type Reminders = HashMap<String, Vec<Reminder>>
