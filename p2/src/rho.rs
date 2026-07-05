/// Brent による Pollard rho 型の因数発見アルゴリズム。
///
/// 入力:
/// - n: 素因数分解対象の正整数
/// - x0: 初期値。0 <= x0 <= n を想定
/// - m: ブロックサイズ。m > 0
/// - f: n を法とする擬似乱数発生関数
///
/// 出力:
/// - Some(d): n の自明でない約数 d
/// - None: 失敗，または入力が不正
pub fn brent_factor<F>(n: i128, x0: i128, m: i128, f: F) -> Option<i128>
where
    F: Fn(i128) -> i128,
{
    // nが1以下なら因数分解できない
    // m が 0 以下ならブロックサイズが不正
    // x0 が 0 未満または n 以上なら初期値が不正
    if n <= 1 || m <= 0 || x0 < 0 || x0 > n {
        return None;
    }

    // 偶数は即座に処理する
    // 因数が2で確定する．
    if n % 2 == 0 {
        return if n == 2 { None } else { Some(2) };
    }

    let mut y = mod_norm(x0, n);
    let mut r: i128 = 1;
    let mut q: i128 = 1;
    let mut g: i128 = 1;

    // g = n になったときの後退探索で使う。
    let mut x: i128 = y;
    let mut ys: i128 = y;

    while g == 1 {
        x = y;

        for _ in 0..r {
            y = mod_norm(f(y), n);
        }

        let mut k: i128 = 0;

        while k < r && g == 1 {
            ys = y;

            let limit = std::cmp::min(m, r - k);

            for _ in 0..limit {
                y = mod_norm(f(y), n);
                let diff = abs_i128(x - y);
                q = mul_mod(q, diff, n);
            }

            g = gcd(q, n);
            k += m;
        }

        // r ← 2r
        r = match r.checked_mul(2) {
            Some(v) => v,
            None => return None,
        };
    }

    // g = n の場合，積 q にまとめたどこかで失敗しているので，
    // ys から 1 ステップずつ戻して単独の gcd を調べる。
    if g == n {
        loop {
            ys = mod_norm(f(ys), n);
            g = gcd(abs_i128(x - ys), n);

            if g > 1 {
                break;
            }
        }
    }

    if g == n { None } else { Some(g) }
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    a = abs_i128(a);
    b = abs_i128(b);

    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }

    a
}

/// 剰余を計算する
/// ただし，剰余が負の数の場合はnを法とした正の整数にする．
fn mod_norm(x: i128, n: i128) -> i128 {
    let r = x % n;
    if r < 0 { r + n } else { r }
}

/// i128 の範囲内で安全に絶対値を取る。
/// このアルゴリズムでは x, y は 0 <= x,y < n を維持するため，
/// x - y が i128::MIN になるケースは通常想定しない。
fn abs_i128(x: i128) -> i128 {
    if x < 0 { -x } else { x }
}

/// (a * b) mod n をオーバーフローせずに計算する。
///
/// a,b,n は非負，n > 0 を想定する。
pub fn mul_mod(mut a: i128, mut b: i128, n: i128) -> i128 {
    a = mod_norm(a, n);
    b = mod_norm(b, n);

    let mut result: i128 = 0;

    while b > 0 {
        if b & 1 == 1 {
            result = add_mod(result, a, n);
        }

        a = add_mod(a, a, n);
        b >>= 1;
    }

    result
}

/// (a + b) mod n をオーバーフローしにくい形で計算する。
///
/// 0 <= a,b < n を想定する。
fn add_mod(a: i128, b: i128, n: i128) -> i128 {
    // a + b >= n を，a >= n - b と同値にして加算前に判定する。
    if a >= n - b { a - (n - b) } else { a + b }
}
