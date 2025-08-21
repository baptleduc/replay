use assert_cmd::Command;
use predicates::prelude::*;
use replay::session::Session;
use serial_test::serial;
use uuid::Uuid;

// Here we will implement the integrations_tests
#[test]
#[serial]
fn test_list_after_drop_session() {
    // We use the user's real session directory for testing,
    // so to avoid collisions with existing sessions,
    // we generate unique random UUID-based descriptions.
    let session_desc1 = &Uuid::new_v4().to_string()[..10];
    let session_desc2 = &Uuid::new_v4().to_string()[..10];

    let session1 = Session::new(Some(session_desc1.to_string())).unwrap();
    session1.save_session(true).unwrap();

    let session2 = Session::new(Some(session_desc2.to_string())).unwrap();
    session2.save_session(true).unwrap();

    let pattern = format!(
        r"(?m)^replay@\{{0\}}: .*, message: {}$",
        // Since we match with regex and UUID contains `-`, need to escape it
        session_desc2.replace("-", r"\-")
    );
    // Verify both sessions appear in `list` output
    Command::cargo_bin("replay")
        .unwrap()
        .arg("list")
        .assert()
        .stdout(
            predicates::str::contains(session_desc1)
                .and(predicates::str::contains(session_desc2))
                .and(predicates::str::is_match(pattern).unwrap()),
        );

    Command::cargo_bin("replay")
        .unwrap()
        .arg("drop")
        .arg("replay@{1}")
        .assert()
        .success();

    // Ensure the first session is no longer listed
    Command::cargo_bin("replay")
        .unwrap()
        .arg("list")
        .assert()
        .stdout(predicates::str::contains(session_desc1).not());

    Command::cargo_bin("replay")
        .unwrap()
        .arg("drop")
        .assert()
        .success();

    Command::cargo_bin("replay")
        .unwrap()
        .arg("list")
        .assert()
        .stdout(predicates::str::contains(session_desc2).not());
}
