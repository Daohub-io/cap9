extern crate assert_cli;

#[cfg(test)]
mod integration {
    use assert_cli;

    #[test]
    fn calling_beaker_without_args() {
        assert_cli::Assert::main_binary()
            .fails()
            .unwrap();
    }

    #[test]
    fn calling_new_example_no_arg() {
        assert_cli::Assert::command(&["cargo", "run", "--", "new"])
            .fails()
            .unwrap();
    }

    #[test]
    fn calling_deploy_example() {
        assert_cli::Assert::command(&["cargo", "run", "--", "deploy"])
            .fails()
            .unwrap();
    }
}
