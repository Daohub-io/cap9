# Getting Started with Cap9

## Creating a New Project

After installing the cap9-cli, run `cap9-cli new <project-name>` to create a new
project. To create a project that includes an ACL, use the `--acl` option.

## Deploying a Kernel

Once a new project has been created, change to the project directory and run
`cap9-cli deploy`. This will deploy the kernel to the blockchain with an initial
entry procedure. If the `--acl` option was selected when building the project
this will also deploy the necessary ACL procedures for a basic system.

## Executing Commands

To see what funcitons are available on an ACL-based system, run `cap9-cli fetch
acl abi`. This will display a list of functions for each group in the ACL and
their type signatures. To execute one of these commands use the `cap9-cli call
<function-name> <inputs...>` command or the `cap9-cli query <function-name>
<inputs...>` to call or query that function respectively.

## Deploying a New Procedure

To simply deploy a procedure to the kernel (without regard for its function
within the ACL) use the `cap9-cli deploy-procedure` with the name of the
procedure (i.e. the procedure key), the path of the binary code file, the path
of the ABI JSON file, and the path of the JSON file containing the capability
information. For example:

```sh
cap9-cli deploy-procedure MembersProc proc_code.bin proc_abi.json caps.json
```

## Adding a New Group to an ACL-Enabled Kernel

To add a new group to an ACL-enabled kernel use the `cap9-cli new-group`
command. The command is run in the following format:

```sh
cap9-cli new-group <GROUP-NUMBER> <PROCEDURE-NAME> <CODE-FILE> <ABI-FILE> <CAP-FILE>
```

where

* GROUP-NUMBER is the number in [1,255] that the group will be assigned to.
* PROCEDURE-NAME is the name/key that the procedure will be registered with.
* CODE-FILE is the file path to the binary code to be deployed.
* ABI-FILE is the JSON ABI file for the given binary code.
* CAP-FILE is the JSON file containing the capability specification.
