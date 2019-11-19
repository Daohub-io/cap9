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
        (call $ext_balance)
        (set_local $balance_size (call $ext_scratch_size))
        (call $ext_scratch_read
            (i32.const 4000)
            (i32.const 0)
            (get_local $balance_size)
        )
        (call $to_hex_ascii
            (i32.const 4000)
            (get_local $balance_size)
            (i32.const 3000)
        )
        (call $ext_println
            (i32.const 3000) ;; The data buffer
            (i32.mul (i32.const 2) (get_local $balance_size)) ;; The data buffer's length
        )
        (call $ext_set_storage
            (i32.const 6000) ;; Pointer to the key
            (i32.const 1)    ;; Value is not null
            (i32.const 32)   ;; Pointer to the value
            (i32.const 1)    ;; Length of the value
        )
        ;; Get the storage value
        (drop (call $ext_get_storage
            (i32.const 6000) ;; Pointer to storage key
        ))
        (set_local $scratch_size_temp (call $ext_scratch_size))
        ;; Read the address from the scratch buffer and store it in memory at
        ;; location 7000.
        (call $ext_scratch_read
            (i32.const 7000) ;; The pointer where to store the data.
            (i32.const 0) ;; Offset from the start of the scratch buffer.
            ;; (i32.const 32) ;; Count of bytes to copy.
            (get_local $scratch_size_temp)
        )
        (call $to_hex_ascii
            (i32.const 7000)
            (get_local $scratch_size_temp)
            (i32.const 8000)
        )
        (call $ext_println
            (i32.const 5000) ;; The data buffer
            (i32.const 23)   ;; The data buffer's length
        )
        (call $ext_println
            (i32.const 8000) ;; The data buffer
            (i32.mul (i32.const 2) (get_local $scratch_size_temp)) ;; The data buffer's length
        )

        ;; Store the address of this contract into the scratch buffer
        (call $ext_address)
        (set_local $address_size (call $ext_scratch_size))
        ;; Read the address from the scratch buffer and store it in memory at
        ;; location 64.
        (call $ext_scratch_read
            (i32.const 64) ;; The pointer where to store the data.
            (i32.const 0) ;; Offset from the start of the scratch buffer.
            ;; (i32.const 32) ;; Count of bytes to copy.
            (get_local $address_size)
        )

        ;; We only call ourselves recursively if a certain value is low, so that we
        ;; don't recurse infintely.
        (if
            (i32.load (i32.const 32))
            (then
                    ;; First, as we are recursing we decrement the recurse counter
                    (i32.store (i32.const 32) (i32.sub (i32.load (i32.const 32)) (i32.const 1)))
                    ;; "calling from..." message
                    (call $ext_println
                        (i32.const 1000) ;; The data buffer
                        (i32.const 15) ;; The data buffer's length
                    )
                    ;; println expects utf8, we can't just send it any bytes. If
                    ;; it is not utf8 it will just skip it.
                    (call $to_hex_ascii
                        (i32.const 64)
                        (get_local $address_size)
                        (i32.const 3000)
                    )
                    (call $ext_println
                        (i32.const 3000) ;; The data buffer
                        (i32.mul (i32.const 2) (get_local $address_size)) ;; The data buffer's length
                    )
                    ;; "calling to..." message
                    (call $ext_println
                        (i32.const 1200) ;; The data buffer
                        (i32.const 13) ;; The data buffer's length
                    )
                    ;; println expects utf8, we can't just send it any bytes. If
                    ;; it is not utf8 it will just skip it.
                    (call $to_hex_ascii
                        (i32.const 1500)
                        (get_local $address_size)
                        (i32.const 3000)
                    )
                    (call $ext_println
                        (i32.const 3000) ;; The data buffer
                        (i32.mul (i32.const 2) (get_local $address_size)) ;; The data buffer's length
                    )

                    (call $ext_call
                        (i32.const 1500) ;; callee_ptr: u32, a pointer to the address of the callee contract. Should be decodable as an `T::AccountId`. Traps otherwise.
                        (i32.const 32) ;; callee_len: u32, length of the address buffer.
                        (i64.const 0) ;; gas: u64, how much gas to devote to the execution (0 = all).
                        ;; IMPORTANT: This was always failing when value wasn't 32 bytes
                        (i32.const 0) ;; value_ptr: u32, a pointer to the buffer with value, how much value to send. Should be decodable as a `T::Balance`. Traps otherwise.
                        (i32.const 32) ;; value_len: u32, length of the value buffer.
                        (i32.const 50) ;; input_data_ptr: u32, a pointer to a buffer to be used as input data to the callee.
                        (i32.const 8) ;; no data sent ;; input_data_len: u32, length of the input data buffer.
                    )
                    i32.const 0
                    i32.store
                    (call $to_hex_ascii
                        (i32.const 0)
                        (i32.const 4)
                        (i32.const 3000)
                    )
                    (call $ext_println
                        (i32.const 3000) ;; The data buffer
                        (i32.const 4) ;; The data buffer's length
                    )
            )
            (else
                ;; If we're finished we just do nothing
            )
        )
    )
    (func (export "deploy"))
    ;; The value we're passing in our call
    (data (i32.const 0) "\00")
    ;; The number of times we will recurse
    (data (i32.const 32) "\02")
    (data (i32.const 1000) "calling from...")
    (data (i32.const 1200) "calling to...")
    (data (i32.const 1500) "\75\42\96\BF\90\25\43\8B\ED\85\16\FB\57\46\7B\A6\1A\58\6C\C1\61\57\2B\13\AA\42\3B\88\C5\51\B6\14")
    ;; byte to hex conversion table
    (data (i32.const 2000) "0123456789ABCDEF")
    (data (i32.const 5000) "Current Storage Value: ")
    ;; Some random storage key (32 bytes)
    (data (i32.const 6000) "\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb\aa\bb")
)
