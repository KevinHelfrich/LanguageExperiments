a = 0 + 5;
zero = 0 + 0;
label loop;
print a;
a = a - 1;
JumpEZ end a;
JumpEZ loop zero;
label end;