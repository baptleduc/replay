use super::RunnableCommand;
use crate::errors::ReplayError;
use crate::session::MetaData;
use crate::session::Session;
use chrono::Utc;
use clap::Args;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct ListCommand {}

impl RunnableCommand for ListCommand {
    fn run(&self) -> Result<(), ReplayError> {
        let list_lines = Self::list()?;
        for line in list_lines {
            let line = line?;
            println!("{}", line);
        }
        Ok(())
    }
}

impl ListCommand {
    pub fn list() -> Result<impl Iterator<Item = Result<String, ReplayError>>, ReplayError> {
        let iter = Session::iter_session_ids_rev()?;
        Ok(iter.enumerate().map(|(i, line_res)| {
            let line = line_res?;
            let session_metadata = Session::load_metadata_by_index(&line)?;
            Ok(format!(
                "replay@{{{}}}: {}",
                i,
                Self::display_metadata(session_metadata)
            ))
        }))
    }
    fn truncate_description(line: &str, max_len: usize) -> String {
        let truncated: String = line.chars().take(max_len).collect();
        if line.chars().count() > max_len {
            truncated + "..."
        } else {
            truncated
        }
    }

    fn display_metadata(metadata: MetaData) -> String {
        if let Some(dess) = metadata.description {
            let list_message = format!(
                "{}, message: {}",
                Self::adapt_date_metadata(metadata.timestamp),
                dess,
            );
            Self::truncate_description(&list_message, 50)
        } else {
            let first_commands_stylized = metadata.first_commands.join(" | ");
            let list_message = format!(
                "{}, commands: {}",
                Self::adapt_date_metadata(metadata.timestamp),
                first_commands_stylized,
            );
            Self::truncate_description(&list_message, 50)
        }
    }

    fn adapt_date_metadata(timestamp: chrono::DateTime<Utc>) -> String {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::tests::setup;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_list_format() {
        setup();
        let mut session_1 = Session::new(None).unwrap();
        session_1.add_command("ls".into());
        session_1.add_command("echo test".into());
        session_1.save_session(true).unwrap();
        let session_2 = Session::new(Some("test session 2".into())).unwrap();
        session_2.save_session(true).unwrap();
        let session_3 = Session::new(Some(
            "session message is too long and should be truncated".into(),
        ))
        .unwrap();
        session_3.save_session(true).unwrap();
        let list_output: Vec<_> = ListCommand::list()
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            format!(
                "replay@{{0}}: {}, message: session message is too lon...",
                ListCommand::adapt_date_metadata(session_3.timestamp),
            ),
            list_output[0]
        );
        assert_eq!(
            format!(
                "replay@{{1}}: {}, message: test session 2",
                ListCommand::adapt_date_metadata(session_2.timestamp)
            ),
            list_output[1]
        );
        assert_eq!(
            format!(
                "replay@{{2}}: {}, commands: ls | echo test",
                ListCommand::adapt_date_metadata(session_1.timestamp)
            ),
            list_output[2]
        );
    }
}
