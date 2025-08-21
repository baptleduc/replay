use assert_cmd::Command;
use predicates::prelude::*;
use replay::session::Session;
use serial_test::serial;
use uuid::Uuid;

// Here we will implement the integrations_tests
#[test]
#[serial]
fn test_list_after_pop_session() {
    let session_name = &Uuid::new_v4().to_string()[..10];
    let session = Session::new(Some(session_name.to_string())).unwrap();
    session.save_session(true).unwrap();

    Command::cargo_bin("replay")
        .unwrap()
        .arg("list")
        .assert()
        .stdout(predicates::str::contains(session_name));

    Command::cargo_bin("replay")
        .unwrap()
        .arg("pop")
        .assert()
        .success();

    Command::cargo_bin("replay")
        .unwrap()
        .arg("list")
        .assert()
        .stdout(predicates::str::contains(session_name).not());
}

#[test]
#[serial]
fn test_list_after_drop_session() {
    let session_name1 = &Uuid::new_v4().to_string()[..10];
    let session_name2 = &Uuid::new_v4().to_string()[..10];

    let session1 = Session::new(Some(session_name1.to_string())).unwrap();
    session1.save_session(true).unwrap();

    let session2 = Session::new(Some(session_name2.to_string())).unwrap();
    session2.save_session(true).unwrap();

    Command::cargo_bin("replay")
        .unwrap()
        .arg("list")
        .assert()
        .stdout(
            predicates::str::contains(session_name1).and(predicates::str::contains(session_name2)),
        );

    Command::cargo_bin("replay")
        .unwrap()
        .arg("drop")
        .arg("replay@{1}")
        .assert()
        .success();

    Command::cargo_bin("replay")
        .unwrap()
        .arg("list")
        .assert()
        .stdout(predicates::str::contains(session_name1).not());

    Command::cargo_bin("replay")
        .unwrap()
        .arg("drop")
        .arg("replay@{0}")
        .assert()
        .success();

    Command::cargo_bin("replay")
        .unwrap()
        .arg("list")
        .assert()
        .stdout(predicates::str::contains(session_name2).not());
}
