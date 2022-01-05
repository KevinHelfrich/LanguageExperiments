b = 5;
while (b > 0) {
    print (b);
    b = b - 1;
}
while (b >= -5) {
    print (b);
    b = b - 1;
}
while (b < 0) {
    print(b);
    b = b + 1;
}
while (b <= 5) {
    print(b);
    b = b + 1;
}
while (b == 6) {
    print(b);
    b = b + 1;
}
while (b != 10) {
    print(b);
    b = b + 1;
    if(b==10){
        println(b);
    }
}

a = 1;
b = 1;

while (b <= 12) {
    while (a <= 12) {
        c = a*b;
        if (a != 12){
            print(c);
        }
        if (a == 12){
            println(c);
        }
        a = a + 1;
    }
    a = 1;
    b = b + 1;
}
