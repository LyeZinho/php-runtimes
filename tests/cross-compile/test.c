#include <stdio.h>

int main() {
    printf("Hello from Windows cross-compilation via Docker!\n");
    printf("Size of int: %zu bytes\n", sizeof(int));
    printf("Size of long: %zu bytes\n", sizeof(long));
    return 0;
}
