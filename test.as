test: MOV A 0b01
JMP #test
JMP #a
ADD         A
JMP         xd A B
a: SUB [0b101] A B
ADD [#a]
