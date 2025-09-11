use super::MetaData;
use chrono::Utc;

/// Small owned wrapper that knows how to format a MetaData for display.
/// We keep it *owned* to simplify usage where metadata comes from an iterator.
pub struct DisplayMeta {
    pub index: usize,
    pub meta: MetaData,
}

impl std::fmt::Display for DisplayMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(desc) = &self.meta.description {
            let list_message = format!(
                "{}, message: {}",
                Self::format_time_ago(self.meta.timestamp),
                desc,
            );
            write!(
                f,
                "replay@{{{}}}: {}",
                self.index,
                Self::truncate_description(&list_message, 50)
            )
        } else {
            let first_commands_stylized = self.meta.first_commands.join(" | ");
            let list_message = format!(
                "{}, commands: {}",
                Self::format_time_ago(self.meta.timestamp),
                first_commands_stylized,
            );
            write!(
                f,
                "replay@{{{}}}: {}",
                self.index,
                Self::truncate_description(&list_message, 50)
            )
        }
    }
}

/// Helpers (kept here for locality).
impl DisplayMeta {
    fn format_time_ago(timestamp: chrono::DateTime<Utc>) -> String {
        let duration = Utc::now().signed_duration_since(timestamp);

        if duration.num_days() > 0 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{} minutes ago", duration.num_minutes())
        } else {
            format!("{} seconds ago", duration.num_seconds())
        }
    }

    fn truncate_description(line: &str, max_len: usize) -> String {
        let truncated: String = line.chars().take(max_len).collect();
        if line.chars().count() > max_len {
            truncated + "..."
        } else {
            truncated
        }
    }
}
