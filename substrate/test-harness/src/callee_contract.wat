;; This contract is called by another contract and should increment a value in
;; storage by 1. The stored value is a single byte, and the address is noted in
;; the data section below.
(module
    (import "env" "ext_scratch_size" (func $ext_scratch_size (result i32)))
    (import "env" "ext_scratch_read" (func $ext_scratch_read (param i32 i32 i32)))
    (import "env" "ext_set_storage" (func $ext_set_storage (param i32 i32 i32 i32)))
    (import "env" "ext_get_storage" (func $ext_get_storage (param i32) (result i32)))
    (import "env" "ext_println" (func $ext_println (param i32 i32)))
    (import "env" "ext_call" (func $ext_call (param i32 i32 i64 i32 i32 i32 i32) (result i32)))
    (import "env" "ext_address" (func $ext_address))
    (import "env" "ext_balance" (func $ext_balance))
    (import "env" "memory" (memory 1 1))

    ;; store_to needs $mem_length*2 bytes
    (func $to_hex_ascii (param $mem_start i32) (param $mem_length i32) (param $store_to i32) (local $i i32) (local $from_addr i32) (local $to_addr i32) (local $char_dict i32)
        (set_local $i (i32.const 0))
        (set_local $char_dict (i32.const 2000))
        (block
            (loop
                (set_local $from_addr (i32.add (get_local $mem_start) (get_local $i)))
                (set_local $to_addr (i32.add (get_local $store_to) (i32.mul (i32.const 2) (get_local $i))))
                ;; store upper four bits
                (i32.store8 (get_local $to_addr) (i32.load (i32.add (get_local $char_dict) (i32.and (i32.const 0x0f) (i32.shr_u (i32.load (get_local $from_addr)) (i32.const 4))))))
                ;; store lower four bits
                (i32.store8 (i32.add (i32.const 1) (get_local $to_addr)) (i32.load (i32.add (get_local $char_dict) (i32.and (i32.const 0x0f) (i32.load (get_local $from_addr))))))

                (set_local $i (i32.add (get_local $i) (i32.const 1)))
                (br_if 1 (i32.ge_u (get_local $i) (get_local $mem_length)))
                (br 0)
            )
        )
    )

    (func (export "call") (local $address_size i32) (local $balance_size i32) (local $scratch_size_temp i32)
        ;; Retrieve the current value from storage (which we will then
        ;; increment). It should be a single byte. If there is currently no
        ;; value this call will return 1.
        (if
            (call $ext_get_storage
                (i32.const 6000) ;; Pointer to storage key
            )
            (then
                ;; There was no value set in storage, so we must set the initial
                ;; value.
                ;; "No value in storage, setting to 1..." message
                (call $ext_println
                    (i32.const 5140) ;; The data buffer
                    (i32.const 45)  ;; The data buffer's length
                )
                ;; Make an 0x01 value in memory.
                (i32.store8 (i32.const 10) (i32.const 1))
                ;; Put the 0x01 value into storage.
                (call $ext_set_storage
                    (i32.const 6000) ;; Pointer to the key
                    (i32.const 1)    ;; Value is not null
                    (i32.const 10)   ;; Pointer to the value
                    (i32.const 1)    ;; Length of the value
                )
                ;; Read the value straight back out of storage.
                (drop (call $ext_get_storage
                    (i32.const 6000) ;; Pointer to storage key
                ))
                (call $ext_scratch_read
                    (i32.const 10) ;; The pointer where to store the data.
                    (i32.const 0) ;; Offset from the start of the scratch buffer.
                    (i32.const 1) ;; Count of bytes to copy.
                )
            )
            (else
                ;; If there is already a value in storage we want to take that
                ;; value and increment it, then store it back.
                (drop (call $ext_get_storage
                    (i32.const 6000) ;; Pointer to storage key
                ))
                (call $ext_scratch_read
                    (i32.const 10) ;; The pointer where to store the data.
                    (i32.const 0) ;; Offset from the start of the scratch buffer.
                    (i32.const 1) ;; Count of bytes to copy.
                )
                ;; Increment the value.
                (i32.store8 (i32.const 10) (i32.add (i32.const 1) (i32.load8_u (i32.const 10))))
                ;; Save the incremented value.
                (call $ext_set_storage
                    (i32.const 6000) ;; Pointer to the key
                    (i32.const 1)    ;; Value is not null
                    (i32.const 10)   ;; Pointer to the value
                    (i32.const 1)    ;; Length of the value
                )
            )
        )
        ;; Read the value back out of storage.
        (drop (call $ext_get_storage
            (i32.const 6000) ;; Pointer to storage key
        ))
        (call $ext_scratch_read
            (i32.const 10) ;; The pointer where to store the data.
            (i32.const 0) ;; Offset from the start of the scratch buffer.
            (i32.const 1) ;; Count of bytes to copy.
        )
        (call $to_hex_ascii
            (i32.const 10)
            (i32.const 1)
            (i32.const 5065)
        )
        (call $ext_println
            (i32.const 5040) ;; The data buffer
            (i32.const 36)   ;; The data buffer's length
        )
    )
    (func (export "deploy"))
    ;; The value we're passing in our call
    (data (i32.const 0) "\00")
    ;; Our temporary allocation for the storage value
    (data (i32.const 10) "\00")
    ;; The number of times we will recurse, we need to have that in storage
    ;; somewhere for it to be useful.
    (data (i32.const 32) "\02")
    (data (i32.const 1000) "[CALLEE] Storing:")
    (data (i32.const 1200) "[CALLEE] To:")
    (data (i32.const 1500) "\75\42\96\BF\90\25\43\8B\ED\85\16\FB\57\46\7B\A6\1A\58\6C\C1\61\57\2B\13\AA\42\3B\88\C5\51\B6\14")
    ;; byte to hex conversion table
    (data (i32.const 2000) "0123456789ABCDEF")
    (data (i32.const 5000) "[CALLEE] Initial Storage Value: 0x")
    (data (i32.const 5040) "[CALLEE] Current Storage Value: 0x")
    (data (i32.const 5140) "[CALLEE] No value in storage, setting to 1...")
    ;; The storage key (32 bytes) which is used in our tests.
    (data (i32.const 6000) "\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb")
)
