extern crate moy_sekret;

use std::fs;
use std::path::Path;
use testaun::testaun_case;

#[macro_use]
pub mod common;
use common::fixtures::*;

// Helpers
//

fn do_something() {}

fn do_something_else() {}

// Test Setup
//

fn testaun_before() {
    do_something();
}

fn testaun_after() {
    do_something_else();
}

// Tests
//

#[test]
#[testaun_case]
#[ignore]
fn should_whatever() {
    assert_eq!("blah", "meh");
}

#[test]
#[testaun_case]
#[ignore]
fn should_whatever_else() {
    assert!(false, "you so wrong");
}
