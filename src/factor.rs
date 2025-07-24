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
