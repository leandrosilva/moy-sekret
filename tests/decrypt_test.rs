extern crate moy_sekret;

use std::fs;
use std::path::Path;

#[macro_use]
pub mod common;
use common::fixtures::*;

// Helpers
//

fn do_something() {
}

fn do_something_else() {
}

// Test Setup
//

setup_run_test!(
    {
        do_something();
    },
    {
        do_something_else();
    }
);

// Tests
//

#[test]
#[ignore]
fn should_whatever() {
    run_test!({
        assert_eq!("blah", "meh");
    })
}

#[test]
#[ignore]
fn should_whatever_else() {
    run_test!({
        assert!(false, "you so wrong");
    })
}
