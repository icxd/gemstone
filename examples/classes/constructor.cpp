#include <stdio.h>
#include <string>
class Foo
{
public:
    Foo(int bar, char *baz) : bar(bar), baz(baz) {}

private:
    int bar;
    char *baz;
    int foo;
};
int main(int argc, char **argv)
{
    return 0;
}
