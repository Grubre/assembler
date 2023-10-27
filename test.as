start:  MOV A 42
        ADD A
        byte 0x05 -0x05 0b001 10 0o12 -10 -0o12
        JMP #start
        a: SUB [0x2A] A B
        b: SUB [#start] B A
        HALT

