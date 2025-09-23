pub const fn is_prime(n: u32) -> bool {
    // every prime > 3 can be written as 6k +/- 1
    if n <= 1 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }
    let mut i: u64 = 5;
    while i * i <= n as u64 {
        if n as u64 % i == 0 || (n as u64) % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}

pub const fn calc_next_prime_not_above(n: u32) -> u32 {
    assert!(n >= 3);
    let mut candidate = n;
    if candidate % 2 == 0 {
        candidate -= 1;
    }
    while !is_prime(candidate) {
        candidate -= 2;
    }
    candidate
}

pub const fn calc_commutative_hash(mut seed: u32, node: usize) -> u32 {
    // since Z_n is a field for prime n, we will never reach 0 except if we start with 0
    if seed == 0 {
        seed += 1;
    }
    // 1000 is the number of cities we expect to handle at most
    // it it's more, the hash will still work, but less efficiently with more collisions
    const PRIME: u32 = calc_next_prime_not_above(u32::MAX / 1000);
    seed.wrapping_mul((node + 1) as u32) % PRIME
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};

    use super::*;

    #[test]
    pub fn test_is_prime() {
        assert!(!is_prime(0));
        assert!(!is_prime(1));
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(!is_prime(4));
        assert!(is_prime(5));
        assert!(is_prime(4294967291));
        assert!(!is_prime(4294967294));
        assert!(!is_prime(4294967295));
    }

    #[test]
    pub fn test_calc_next_prime_not_above() {
        assert_eq!(calc_next_prime_not_above(10), 7);
        assert_eq!(calc_next_prime_not_above(11), 11);
        assert_eq!(calc_next_prime_not_above(12), 11);
        assert_eq!(calc_next_prime_not_above(13), 13);
        assert_eq!(calc_next_prime_not_above(14), 13);
        assert_eq!(calc_next_prime_not_above(15), 13);
        assert_eq!(calc_next_prime_not_above(16), 13);
        assert_eq!(calc_next_prime_not_above(17), 17);
        assert_eq!(calc_next_prime_not_above(4294967295), 4294967291);
    }

    #[test]
    pub fn test_calc_commutative_hash_is_never_zero() {
        let mut hash = 1;
        for i in 0..1000 {
            hash = calc_commutative_hash(hash, i);
            assert!(hash != 0);
        }
    }

    #[test]
    pub fn test_calc_commutative_hash_is_commutative() {
        let mut hash_forward = 1;
        for i in 1..1000 {
            // println!("i = {i}, hash = {hash_forward}");
            hash_forward = calc_commutative_hash(hash_forward, i);
        }

        let mut hash_backward = 1;
        for i in (1..1000).rev() {
            hash_backward = calc_commutative_hash(hash_backward, i);
        }
        assert_eq!(hash_forward, hash_backward);

        let mut rng = StdRng::seed_from_u64(42);

        let mut indices = (0..1000).collect::<Vec<usize>>();

        for _ in 1..10 {
            indices.shuffle(&mut rng);

            let mut hash_shuffled = 0;
            for node in &indices {
                hash_shuffled = calc_commutative_hash(hash_shuffled, *node);
            }
            assert_eq!(hash_backward, hash_shuffled);
        }
    }
}
