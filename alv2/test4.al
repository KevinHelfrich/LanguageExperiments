a[] = 2 , 3 ,4, 5, 6 ,7;
a[0] = 1;
b = a[0];
println(b);
c = 0;
while (c < 6) {
    print(a[c]);
    c = c + 1;
}
c = 12;
a[c] = 15;
println(a[c]);
