// Macros
//

#[macro_export]
macro_rules! setup_run_test {
    ($before_test:block, $after_test:block) => {
        use std::panic;

        #[macro_export]
        macro_rules! run_test {
            ($test_exp:block) => {{
                $before_test;
                let result = panic::catch_unwind(|| $test_exp);
                $after_test;
                assert!(result.is_ok());
            }};
        }
    };
}

// Fixtures
//

pub mod fixtures {
    pub const F_STORAGE_DIR: &str = "./tests_temp";
    pub const F_PROFILE: &str = "int_tester";
    pub const F_OVERRIDE_PROFILE: bool = false;
}
