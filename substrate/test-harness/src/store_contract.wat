(module
    (import "env" "ext_scratch_size" (func $ext_scratch_size (result i32)))
    (import "env" "ext_scratch_read" (func $ext_scratch_read (param i32 i32 i32)))
    (import "env" "ext_set_storage" (func $ext_set_storage (param i32 i32 i32 i32)))
    (import "env" "ext_get_storage" (func $ext_get_storage (param i32) (result i32)))
    (import "env" "ext_println" (func $ext_println (param i32 i32)))
    (import "env" "memory" (memory 1 1))
    (func (export "call")
        (call $ext_set_storage
            (i32.const 0) ;; key_ptr: Pointer to the key
            (i32.const 1)  ;; value_non_null: Clearance flag - if zero clear storage
            (i32.const 6) ;; value_ptr: Pointer to the start of the value
            (i32.const 3)  ;; value_len: Length of the value
        )
        ;; Return value is 0 for success, simply drop this and assume success
        (drop (call $ext_get_storage
            (i32.const 0) ;; Pointer to the key
        ))
        ;;(i32.store
        ;;    (i32.const 32)
        ;;    (call $ext_get_storage
        ;;        (i32.const 0) ;; Pointer to the key
        ;;    )
        ;;)
        (call $ext_scratch_read
            (i32.const 32) ;; The pointer where to store the data.
            (i32.const 0) ;; Offset from the start of the scratch buffer.
            (i32.const 3) ;; Count of bytes to copy.
        )
        (call $ext_println
            (i32.const 32) ;; The data buffer
            (i32.const 3) ;; The data buffer's length
        )
    )
    (func (export "deploy"))
    ;; The storage keys
    (data (i32.const 0) "\01\ab\cd\ef")
    ;; The value to store
    (data (i32.const 6) "abc")
)
