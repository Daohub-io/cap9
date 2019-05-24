;; sample
(module
  (type $t0 (func (param i32 i32) (result i32)))
  (type $t1 (func (param i32 i32 i32 i32)))
  (type $t2 (func (param i32) (result i32)))
  (type $t3 (func (param i32 i32 i32) (result i32)))
  (type $t4 (func (param i32 i32)))
  (type $t5 (func (param i32)))
  (type $t6 (func (result i32)))
  
  (import "env" "storage_write" (func $env.storage_write (type $t4)))
  (import "env" "sender" (func $env.sender (type $t5)))
  (import "env" "value" (func $env.value (type $t5)))
  (import "env" "input_length" (func $env.input_length (type $t6)))
  (import "env" "fetch_input" (func $env.fetch_input (type $t5)))
  (import "env" "panic" (func $env.panic (type $t4)))
  (import "env" "memory" (memory $env.memory 2 16))
  (import "env" "ret" (func $env.ret (type $t4)))

  (func (export "call")
    i32.const 69344
    i32.const 19805
    call $env.ret)

)