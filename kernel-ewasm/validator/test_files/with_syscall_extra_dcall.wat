;;; This is a non-compliant contract with a syscall, but also an extra dcall
;;; that needs to be caught, as it is not in a syscall compliant position.
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
    i64.const 5
    i32.const 6
    i32.const 7
    i32.const 8
    i32.const 9
    i32.const 10
    i32.const 11
    ;; This is the non-compliant dcall, the preceeding 6 constants are just
    ;; dummy inputs to pass validation.
    call $env.dcall
    drop
    unreachable)
  ;; This is our system call which we have statically linked in
  (func $syscall (type $t3) (param $p0 i32) (param $p1 i32) (param $p2 i32) (param $p3 i32) (result i32)
    call $env.gasleft
    call $env.sender
    get_local $p0
    get_local $p1
    get_local $p2
    get_local $p3
    call $env.dcall)
  (table $T0 17 17 anyfunc)
  (memory $M0 2)
  (global $g0 (mut i32) (i32.const 65536))
  (export "call" (func $call)))
