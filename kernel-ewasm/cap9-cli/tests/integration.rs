#[cfg(test)]
mod integration {
    use std::process::Command;
    use assert_cmd::prelude::*;

    #[test]
    fn calling_cli_without_args() {
        let mut cmd = Command::cargo_bin("cap9-cli").unwrap();
        cmd.assert().failure();
    }

    #[test]
    fn calling_new_example_no_arg() {
        let mut cmd = Command::cargo_bin("cap9-cli").unwrap();
        cmd
            .arg("new");
        cmd.assert().failure();
    }

    #[test]
    fn create_and_deploy() {
        use tempfile::tempdir;

        let project_name = "example";

        // Create a directory inside the temporary directory of the system.
        let dir = tempdir().unwrap();

        // Create a new project
        let mut create_cmd = Command::cargo_bin("cap9-cli").unwrap();
        create_cmd
            .arg("new")
            .arg(project_name)
            .current_dir(dir.path());
        create_cmd.assert().success();

        let mut project_dir = std::path::PathBuf::new();
        project_dir.push(dir.path());
        project_dir.push(project_name);

        // Deploy the kernel
        println!("Deploying project");
        let mut deploy_cmd = Command::cargo_bin("cap9-cli").unwrap();
        deploy_cmd
            .arg("deploy")
            .current_dir(project_dir);
        deploy_cmd.assert().success();

        // Explicity close the temp directory.
        dir.close().unwrap();
    }
}
