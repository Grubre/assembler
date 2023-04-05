test: MOV A 0b01
JMP #test
JMP #a
ADD A ; f
a: SUB [0b101] A B
ADD [#b]
