start:  MOV A 42
        ADD A
        byte 0x05 0x05 0b001 10 012 10 012
        JMPIMMNZ #start
        a: SUB [#b] A B
        b: SUB [#b] B A
        ADD [#hehe]
        HALT

hehe: byte 0x05
