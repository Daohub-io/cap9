#[cfg(test)]
mod integration {
    use assert_cmd::prelude::*;
    use cap9_cli::connection;
    use cap9_cli::fetch::{DeployedKernel, DeployedKernelWithACL};
    use cap9_cli::project;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::tempdir;
    use tempfile::TempDir;

    #[test]
    fn calling_cli_without_args() {
        let mut cmd = Command::cargo_bin("cap9-cli").unwrap();
        cmd.assert().failure();
    }

    #[test]
    fn calling_new_example_no_arg() {
        let mut cmd = Command::cargo_bin("cap9-cli").unwrap();
        cmd.arg("new");
        cmd.assert().failure();
    }

    fn create_temp_project_with_acl() -> (TempDir, PathBuf) {
        let project_name = "example";

        // Create a directory inside the temporary directory of the system.
        let dir = tempdir().unwrap();

        // Create a new project
        let mut create_cmd = Command::cargo_bin("cap9-cli").unwrap();
        create_cmd
            .arg("new")
            .arg("--acl")
            .arg(project_name)
            .current_dir(dir.path());
        create_cmd.assert().success();

        let mut project_dir = PathBuf::new();
        project_dir.push(dir.path());
        project_dir.push(project_name);

        // Deploy the kernel
        println!("Deploying project");
        let mut deploy_cmd = Command::cargo_bin("cap9-cli").unwrap();
        deploy_cmd.arg("deploy").current_dir(&project_dir);
        deploy_cmd.assert().success();
        (dir, project_dir)
    }

    #[test]
    fn create_and_deploy() {
        let project_name = "example";

        // Create a directory inside the temporary directory of the system.
        let dir = tempdir().unwrap();

        // Create a new project
        let mut create_cmd = Command::cargo_bin("cap9-cli").unwrap();
        create_cmd
            .arg("new")
            .arg("--acl")
            .arg(project_name)
            .current_dir(dir.path());
        create_cmd.assert().success();

        let mut project_dir = std::path::PathBuf::new();
        project_dir.push(dir.path());
        project_dir.push(project_name);

        // Deploy the kernel
        println!("Deploying project");
        let mut deploy_cmd = Command::cargo_bin("cap9-cli").unwrap();
        deploy_cmd.arg("deploy").current_dir(&project_dir);
        deploy_cmd.assert().success();

        // There should be one group (1)
        let conn: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        // Read the local project from out current directory.
        let local_project = project::LocalProject::read_dir(&project_dir);
        let kernel = DeployedKernel::new(&conn, local_project);
        let kernel_with_acl = DeployedKernelWithACL::new(kernel);
        let groups_1 = kernel_with_acl.groups();
        assert_eq!(
            groups_1.len(),
            1,
            "There should be one group, but there are {}",
            groups_1.len()
        );

        let res = Command::cargo_bin("cap9-cli")
            .unwrap()
            .arg("fetch")
            .arg("acl")
            .arg("groups")
            .current_dir(&project_dir)
            .assert()
            .success();
        let out = res.get_output();
        println!("out: {}", String::from_utf8(out.stdout.clone()).unwrap());

        let wasm_path: PathBuf = [&project_dir, &PathBuf::from("acl_group_5.wasm")]
            .iter()
            .collect();
        let json_path: PathBuf = [&project_dir, &PathBuf::from("ACLGroup5Interface.json")]
            .iter()
            .collect();
        let caps_path: PathBuf = [&project_dir, &PathBuf::from("example_caps.json")]
            .iter()
            .collect();
        std::fs::copy(PathBuf::from("src/lib/acl_group_5.wasm"), wasm_path).unwrap();
        std::fs::copy(PathBuf::from("src/lib/ACLGroup5Interface.json"), json_path).unwrap();
        std::fs::copy(PathBuf::from("src/lib/example_caps.json"), caps_path).unwrap();
        println!("files copied",);
        // Add a new group to the kernel
        Command::cargo_bin("cap9-cli")
            .unwrap()
            // The command
            .arg("new-group")
            // The number/id of the group
            .arg("5")
            // The name of the group's procedure
            .arg("randomProcName")
            // The file path of the binary code
            .arg("acl_group_5.wasm")
            // The file path of the JSON ABI
            .arg("ACLGroup5Interface.json")
            // The file path of the JSON capability file
            .arg("example_caps.json")
            .current_dir(&project_dir)
            .assert()
            .success();

        let groups_2 = kernel_with_acl.groups();
        assert_eq!(
            groups_2.len(),
            2,
            "There should be one group, but there are {}",
            groups_2.len()
        );

        {
            // There should be 3 procedures:
            //   1. Entry
            //   2. Admin
            //   3. Group 5
            let procedures = kernel_with_acl.kernel.procedures();
            assert_eq!(
                procedures.len(),
                3,
                "There should be 3 procedures, but there are {}",
                procedures.len()
            );
        }

        // Add a new procedure to the kernel
        Command::cargo_bin("cap9-cli")
            .unwrap()
            // The command
            .arg("deploy-procedure")
            // The name of the group's procedure
            .arg("anotherProcName")
            // The file path of the binary code
            .arg("acl_group_5.wasm")
            // The file path of the JSON ABI
            .arg("ACLGroup5Interface.json")
            // The file path of the caps file
            .arg("example_caps.json")
            .current_dir(&project_dir)
            .assert()
            .success();

        {
            // There should be 4 procedures:
            //   1. Entry
            //   2. Admin
            //   3. Group 5
            //   4. New Procedure
            let procedures = kernel_with_acl.kernel.procedures();
            assert_eq!(
                procedures.len(),
                4,
                "There should be 4 procedures, but there are {}",
                procedures.len()
            );
        }
    }

    #[test]
    fn deploy_proc_once() {
        let (_dir, project_dir) = create_temp_project_with_acl();
        let wasm_path: PathBuf = [&project_dir, &PathBuf::from("acl_group_5.wasm")]
            .iter()
            .collect();
        let json_path: PathBuf = [&project_dir, &PathBuf::from("ACLGroup5Interface.json")]
            .iter()
            .collect();
        let caps_path: PathBuf = [&project_dir, &PathBuf::from("example_caps.json")]
            .iter()
            .collect();
        std::fs::copy(PathBuf::from("src/lib/acl_group_5.wasm"), wasm_path).unwrap();
        std::fs::copy(PathBuf::from("src/lib/ACLGroup5Interface.json"), json_path).unwrap();
        std::fs::copy(PathBuf::from("src/lib/example_caps.json"), caps_path).unwrap();
        // Add a new procedure to the kernel
        Command::cargo_bin("cap9-cli")
            .unwrap()
            // The command
            .arg("deploy-procedure")
            // The name of the group's procedure
            .arg("anotherProcName")
            // The file path of the binary code
            .arg("acl_group_5.wasm")
            // The file path of the JSON ABI
            .arg("ACLGroup5Interface.json")
            // The file path of the caps file
            .arg("example_caps.json")
            .current_dir(&project_dir)
            .assert()
            .success();
    }

    /// This test demonstrates that a procedure cannot be replaced or have its
    /// caps updated simply.
    #[test]
    #[should_panic]
    fn deploy_proc_twice() {
        let (_dir, project_dir) = create_temp_project_with_acl();

        let wasm_path: PathBuf = [&project_dir, &PathBuf::from("acl_group_5.wasm")]
            .iter()
            .collect();
        let json_path: PathBuf = [&project_dir, &PathBuf::from("ACLGroup5Interface.json")]
            .iter()
            .collect();
        let caps_path: PathBuf = [&project_dir, &PathBuf::from("example_caps.json")]
            .iter()
            .collect();
        std::fs::copy(PathBuf::from("src/lib/acl_group_5.wasm"), wasm_path).unwrap();
        std::fs::copy(PathBuf::from("src/lib/ACLGroup5Interface.json"), json_path).unwrap();
        std::fs::copy(PathBuf::from("src/lib/example_caps.json"), caps_path).unwrap();
        // Add a new procedure to the kernel
        Command::cargo_bin("cap9-cli")
            .unwrap()
            // The command
            .arg("deploy-procedure")
            // The name of the group's procedure
            .arg("anotherProcName")
            // The file path of the binary code
            .arg("acl_group_5.wasm")
            // The file path of the JSON ABI
            .arg("ACLGroup5Interface.json")
            // The file path of the caps file
            .arg("example_caps.json")
            .current_dir(&project_dir)
            .assert()
            .success();
        Command::cargo_bin("cap9-cli")
            .unwrap()
            // The command
            .arg("deploy-procedure")
            // The name of the group's procedure
            .arg("anotherProcName")
            // The file path of the binary code
            .arg("acl_group_5.wasm")
            // The file path of the JSON ABI
            .arg("ACLGroup5Interface.json")
            // The file path of the caps file
            .arg("example_caps.json")
            .current_dir(&project_dir)
            .assert()
            .success();
    }

    /// This test demonstrates that a procedure can be replaced if it is deleted
    /// first.
    #[test]
    fn deploy_proc_twice_with_delete() {
        let (_dir, project_dir) = create_temp_project_with_acl();
        let conn: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        // Read the local project from out current directory.
        let local_project = project::LocalProject::read_dir(&project_dir);
        let kernel = DeployedKernel::new(&conn, local_project);
        let kernel_with_acl = DeployedKernelWithACL::new(kernel);

        let wasm_path: PathBuf = [&project_dir, &PathBuf::from("acl_group_5.wasm")]
            .iter()
            .collect();
        let json_path: PathBuf = [&project_dir, &PathBuf::from("ACLGroup5Interface.json")]
            .iter()
            .collect();
        let caps_path: PathBuf = [&project_dir, &PathBuf::from("example_caps.json")]
            .iter()
            .collect();
        std::fs::copy(PathBuf::from("src/lib/acl_group_5.wasm"), wasm_path).unwrap();
        std::fs::copy(PathBuf::from("src/lib/ACLGroup5Interface.json"), json_path).unwrap();
        std::fs::copy(PathBuf::from("src/lib/example_caps.json"), caps_path).unwrap();
        let procedures1 = kernel_with_acl.kernel.procedures();
        // Add a new procedure to the kernel
        Command::cargo_bin("cap9-cli")
            .unwrap()
            // The command
            .arg("deploy-procedure")
            // The name of the group's procedure
            .arg("anotherProcName")
            // The file path of the binary code
            .arg("acl_group_5.wasm")
            // The file path of the JSON ABI
            .arg("ACLGroup5Interface.json")
            // The file path of the caps file
            .arg("example_caps.json")
            .current_dir(&project_dir)
            .assert()
            .success();
        let procedures2 = kernel_with_acl.kernel.procedures();
        assert_eq!(procedures2.len(), procedures1.len() + 1);
        // Delete the original procedure.
        Command::cargo_bin("cap9-cli")
            .unwrap()
            // The command
            .arg("delete-procedure")
            // The name of the group's procedure
            .arg("anotherProcName")
            .current_dir(&project_dir)
            .assert()
            .success();
        let procedures3 = kernel_with_acl.kernel.procedures();
        assert_eq!(procedures3.len(), procedures1.len());
        Command::cargo_bin("cap9-cli")
            .unwrap()
            // The command
            .arg("deploy-procedure")
            // The name of the group's procedure
            .arg("anotherProcName")
            // The file path of the binary code
            .arg("acl_group_5.wasm")
            // The file path of the JSON ABI
            .arg("ACLGroup5Interface.json")
            // The file path of the caps file
            .arg("example_caps.json")
            .current_dir(&project_dir)
            .assert()
            .success();
        let procedures4 = kernel_with_acl.kernel.procedures();
        assert_eq!(procedures4.len(), procedures1.len() + 1);
    }
}
