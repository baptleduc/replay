use super::RunnableCommand;
use crate::errors::ReplayResult;
use crate::session::DisplayMeta;
use crate::session::Session;
use clap::Args;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct ListCommand {}

impl RunnableCommand for ListCommand {
    fn run(&self) -> ReplayResult<()> {
        for session_infos in Self::list()? {
            println!("{}", session_infos?)
        }
        Ok(())
    }
}

impl ListCommand {
    fn list() -> ReplayResult<impl Iterator<Item = ReplayResult<String>>> {
        Ok(Session::get_all_session_metadata()?.enumerate().map(
            |(i, metadata)| -> ReplayResult<String> {
                let md = metadata?;
                Ok(DisplayMeta { index: i, meta: md }.to_string())
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::tests::setup;
    use regex::Regex;
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
        let re1 = Regex::new(
            r"^replay@\{0\}: \d+ seconds ago, message: session message is too lon\.\.\.$",
        )
        .unwrap();
        assert!(re1.is_match(&list_output[0]));
        let re2 = Regex::new(r"^replay@\{1\}: \d+ seconds ago, message: test session 2$").unwrap();
        assert!(re2.is_match(&list_output[1]));
        let re3 = Regex::new(r"^replay@\{2\}: \d+ seconds ago, commands: ls | echo test$").unwrap();
        assert!(re3.is_match(&list_output[2]));
    }
}
