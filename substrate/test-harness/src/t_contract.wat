(module
    (import "env" "cap9_clist" (func $cap9_clist))
    (import "env" "ext_println" (func $ext_println (param i32 i32)))
    (import "env" "ext_scratch_size" (func $ext_scratch_size (result i32)))
    (import "env" "ext_scratch_read" (func $ext_scratch_read (param i32 i32 i32)))
    ;; env.println
    (import "env" "memory" (memory 1 1))
    (func (export "call") (local $clist_length i32) (local $i i32) (local $addr i32)
        call $cap9_clist
        i32.const 1 ;; The pointer where to store the data.
        i32.const 0 ;; Offset from the start of the scratch buffer.
        call $ext_scratch_size ;; Count of bytes to copy.
        set_local $clist_length
        get_local $clist_length
        call $ext_scratch_read
        ;; For each of our bytes (7) we copy either "yes" or "no" into the
        ;; relevant string
        (set_local $i (i32.const 0))
        (block
            (loop
                (set_local $addr (i32.add (i32.const 300) (i32.mul (get_local $i) (i32.const 30))))
                (if
                    (i32.load8_u
                        (i32.add
                            (i32.const 1)
                            (get_local $i)
                        )
                    )
                    (then
                        (i32.store (i32.add (get_local $addr) (i32.const 19)) (i32.load (i32.const 256)))
                    )
                    (else
                        (i32.store (i32.add (get_local $addr) (i32.const 19)) (i32.load (i32.const 260)))
                    )
                )
                (call $ext_println
                    (get_local $addr) ;; The data buffer
                    (i32.const 23) ;; The data buffer's length
                )
                (set_local $i (i32.add (get_local $i) (i32.const 1)))
                (br_if 1 (i32.ge_u (get_local $i) (get_local $clist_length)))
                (br 0)
            )
        )
    )
    (func (export "deploy"))
    (data (i32.const 256) " yes")
    (data (i32.const 260) "  no")
    (data (i32.const 300) "ProcedureCall    : ")
    (data (i32.const 330) "ProcedureRegister: ")
    (data (i32.const 360) "ProcedureDelete  : ")
    (data (i32.const 390) "ProcedureEntry   : ")
    (data (i32.const 420) "StoreWrite       : ")
    (data (i32.const 450) "Log              : ")
    (data (i32.const 480) "AccountCall      : ")
)
