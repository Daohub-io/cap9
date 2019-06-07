;;; This is a non-compliant contract with a syscall that is similar to the one
;;; specified but not quite (it has a few instructions added). This is the same
;;; as "with_syscall_compliant.wat" except for those 2 instructions. See the
;;; $syscall function below for details.
(module
  (type $t0 (func (param i32 i32)))
  (type $t3 (func (param i32 i32 i32 i32) (result i32)))
  (type $t4 (func (param i64 i32 i32 i32 i32 i32) (result i32)))
  (type $t6 (func (result i64)))
  (type $t7 (func (result i32)))
  (type $t8 (func (result i32)))
  (type $t9 (func))
  (import "env" "dcall" (func $env.dcall (type $t4)))
  (import "env" "gasleft" (func $env.gasleft (type $t6)))
  (import "env" "sender" (func $env.sender (type $t7)))
  (import "env" "input_length" (func $env.input_length (type $t8)))
  (import "env" "fetch_input" (func $env.fetch_input (type $t7)))
  (import "env" "ret" (func $env.ret (type $t0)))
  ;; This is the entry point of the contract
  (func $call (type $t9)
    i32.const 5
    i32.const 3
    i32.add
    drop
    unreachable)
  ;; This is our system call which we have statically linked in
  (func $syscall (type $t3) (param $p0 i32) (param $p1 i32) (param $p2 i32) (param $p3 i32) (result i32)
    call $env.gasleft
    call $env.sender
    get_local $p0
    get_local $p1
    ;; These next two instruction have no effect, but make the syscall
    ;; non-compliant
    i32.const 5
    drop
    get_local $p2
    get_local $p3
    call $env.dcall)
  (table $T0 17 17 anyfunc)
  (memory $M0 2)
  (global $g0 (mut i32) (i32.const 65536))
  (export "call" (func $call)))
