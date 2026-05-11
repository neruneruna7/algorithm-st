// バブルソート
// 1番地にnが当てられていて，20番地からn+19番地までにn個の整数が与えられている
int main() {
    // RAM想定のメモリ
    int n = 20;
    int mem[20] = {0, 5, 3, 8, 1, 4, 7, 2, 6, 9, 10, 15, 12, 11, 14, 13, 19, 18, 17, 16};

    // 入力
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n - i - 1; j++) {
            if (mem[j] > mem[j + 1]) {
                // swap
                int temp = mem[j];
                mem[j] = mem[j+1];
                mem[j+1] = temp;
            }
        }
    }
}