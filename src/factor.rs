use std::io::{self, BufRead, Write};

pub fn run(args: &[String]) -> i32 {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut exit_code = 0;

    if args.is_empty() {
        // Read from stdin
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(l) => {
                    for tok in l.split_whitespace() {
                        exit_code |= factor_one(tok, &mut out);
                    }
                }
                Err(e) => { eprintln!("factor: {}", e); exit_code = 1; }
            }
        }
    } else {
        for arg in args {
            exit_code |= factor_one(arg, &mut out);
        }
    }
    exit_code
}

fn factor_one<W: Write>(s: &str, out: &mut W) -> i32 {
    let n: u64 = match s.trim().parse() {
        Ok(v) => v,
        Err(_) => { eprintln!("factor: '{}' is not a valid positive integer", s); return 1; }
    };
    let factors = factorize(n);
    let _ = write!(out, "{}:", n);
    for f in &factors { let _ = write!(out, " {}", f); }
    let _ = writeln!(out);
    0
}

fn factorize(mut n: u64) -> Vec<u64> {
    let mut factors = Vec::new();
    if n <= 1 { factors.push(n); return factors; }

    // Trial division up to sqrt, then Pollard's rho for large composites
    let small_primes = [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    for &p in &small_primes {
        while n % p == 0 { factors.push(p); n /= p; }
    }
    if n == 1 { return factors; }

    // Trial division up to sqrt(n) or 1000
    let mut d = 41u64;
    while d * d <= n && d < 1000 {
        while n % d == 0 { factors.push(d); n /= d; }
        d += 2;
        while n % d == 0 { factors.push(d); n /= d; }
        d += 4;
    }

    if n > 1 {
        // Use Pollard's rho for the remainder
        let mut stack = vec![n];
        while let Some(m) = stack.pop() {
            if m == 1 { continue; }
            if is_prime(m) {
                factors.push(m);
            } else {
                let f = pollard_rho(m);
                stack.push(f);
                stack.push(m / f);
            }
        }
    }

    factors.sort_unstable();
    factors
}

fn is_prime(n: u64) -> bool {
    if n < 2 { return false; }
    if n == 2 || n == 3 || n == 5 || n == 7 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    // Miller-Rabin with deterministic witnesses for n < 3,317,044,064,679,887,385,961,981
    let witnesses: &[u64] = &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    let (mut d, mut r) = (n - 1, 0u32);
    while d % 2 == 0 { d /= 2; r += 1; }
    'outer: for &a in witnesses {
        if a >= n { continue; }
        let mut x = mod_pow(a, d, n);
        if x == 1 || x == n - 1 { continue; }
        for _ in 0..r - 1 {
            x = mul_mod(x, x, n);
            if x == n - 1 { continue 'outer; }
        }
        return false;
    }
    true
}

fn pollard_rho(n: u64) -> u64 {
    if n % 2 == 0 { return 2; }
    let mut rng = n;
    loop {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let c = (rng % (n - 1)) + 1;
        let mut x = (rng >> 33) % n + 2;
        let mut y = x;
        let mut d = 1u64;
        while d == 1 {
            x = (mul_mod(x, x, n) + c) % n;
            y = (mul_mod(y, y, n) + c) % n;
            y = (mul_mod(y, y, n) + c) % n;
            d = gcd(x.abs_diff(y), n);
        }
        if d != n { return d; }
    }
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 { let t = b; b = a % b; a = t; }
    a
}

fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    let mut result = 1u64;
    base %= modulus;
    while exp > 0 {
        if exp & 1 == 1 { result = mul_mod(result, base, modulus); }
        exp >>= 1;
        base = mul_mod(base, base, modulus);
    }
    result
}

fn mul_mod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}
