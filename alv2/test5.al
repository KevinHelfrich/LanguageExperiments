me() => {
    b = 321;
}
hello(a, c) => {
    a = c * 3;
}
me();
b = 0;
hello(b, 42);
println(b);
fib(fib, n, res) => {
    if(n<=1){
        res = 1;
    }
    if(n>1){
        e = n-1;
        g = n-2;
        f = 0;
        d = 0;
        fib(fib, e, f);
        fib(fib, g, d);
        res = f + d;
    }
}

h = 0;
f = 0;
d = 0;
fib(fib, 9, h);
println(h);



