#include <stdio.h>

int main(void) {
    int n = 0;
    int a, b, c, d = 0;


    for (a = 0; a * a <= n; a++) {
        int n1 = n - a * a;

        for (b = a; b * b <= n1; b++) {
            int n2 = n1 - b * b;

            for (c = b; c * c <= n2; c++) {
                int n3 = n2 - c * c;

                for (d = c; d * d <= n3; d++) {
                    if (d * d == n3) {
                        printf("%d\n", a);
                        printf("%d\n", b);
                        printf("%d\n", c);
                        printf("%d\n", d);
                        return 0;
                    }
                }
            }
        }
    }

    return 1;
}