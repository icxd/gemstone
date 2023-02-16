#include <stdio.h>
#include <string>
class Parent {
public:
virtual int foo() {
printf("Parent::foo()\n");
return 123;
}
private:
virtual int bar() = 0;
};
class Child: public Parent {
public:
int foo() override {
printf("Child::foo()\n");
return 456;
}
private:
int bar() override {
printf("Child::bar()\n");
return 789;
}
};
int main(int argc, char** argv) {
Parent* p = new Child();
Child* c = new Child();
int const foo = p->foo();
return 0;
}
