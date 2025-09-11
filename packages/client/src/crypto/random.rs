//! Random number generation utilities

// xor-shift
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn fast_random() -> u64 {
    use std::cell::Cell;
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    use std::num::Wrapping;

    thread_local! {
        static RNG: Cell<Wrapping<u64>> = Cell::new(Wrapping(seed()));
    }

    fn seed() -> u64 {
        let seed = RandomState::new();

        let mut out = 0;
        let mut cnt = 0;
        while out == 0 {
            cnt += 1;
            let mut hasher = seed.build_hasher();
            hasher.write_usize(cnt);
            out = hasher.finish();
        }
        out
    }

    RNG.with(|rng| {
        let mut n = rng.get();
        debug_assert_ne!(n.0, 0);
        n ^= n >> 12;
        n ^= n << 25;
        n ^= n >> 27;
        rng.set(n);
        n.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    })
}

/// WASM-compatible fast random using web crypto API when available
#[cfg(target_arch = "wasm32")]
pub(crate) fn fast_random() -> u64 {
    use std::cell::Cell;
    use std::num::Wrapping;

    thread_local! {
        static RNG: Cell<Wrapping<u64>> = Cell::new(Wrapping(wasm_seed()));
    }

    fn wasm_seed() -> u64 {
        // Try to get random seed from web crypto API if available
        if let Some(crypto) = web_sys::window().and_then(|w| w.crypto().ok()) {
            let mut array = [0u8; 8];
            if crypto.get_random_values_with_u8_array(&mut array).is_ok() {
                return u64::from_le_bytes(array);
            }
        }
        
        // Fallback to timestamp-based seeding
        let now = js_sys::Date::now();
        let mut seed = now as u64;
        
        // Add some entropy from performance timing if available
        if let Some(perf) = web_sys::window().and_then(|w| w.performance()) {
            let timing = perf.now();
            seed ^= (timing * 1000000.0) as u64;
        }
        
        // Ensure non-zero
        if seed == 0 {
            seed = 0x853c49e6_748fea9b;  // Fixed fallback
        }
        
        seed
    }

    RNG.with(|rng| {
        let mut n = rng.get();
        debug_assert_ne!(n.0, 0);
        // Same xor-shift algorithm as native version
        n ^= n >> 12;
        n ^= n << 25;
        n ^= n >> 27;
        rng.set(n);
        n.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    })
}

/// Generate boundary string for multipart forms
#[must_use] 
pub fn generate_boundary() -> String {
    format!("----formdata-fluent-{:016x}", fast_random())
}

/// Generate nonce for authentication
#[must_use] 
pub fn generate_nonce() -> String {
    format!("{:016x}", fast_random())
}
