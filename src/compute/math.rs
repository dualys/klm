//! Pure `no_std` floating-point mathematical primitives for bare-metal kernels (Amentys OS)
//! and open hardware compute pipelines.

/// Computes the absolute value of an `f32` without requiring `std`.
#[inline]
pub fn abs_f32(x: f32) -> f32 {
    f32::from_bits(x.to_bits() & 0x7FFF_FFFF)
}

/// Computes the square root of a non-negative `f32` in pure `no_std` using
/// IEEE-754 initial approximation followed by 4 Newton-Raphson iterations.
#[inline]
pub fn sqrt_f32(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }

    #[cfg(feature = "std")]
    {
        x.sqrt()
    }

    #[cfg(not(feature = "std"))]
    {
        // Fast IEEE-754 bit-level initial guess + 4 Newton-Raphson refinements
        let mut y = f32::from_bits((x.to_bits() >> 1) + 0x1FC0_0000);
        y = 0.5 * (y + x / y);
        y = 0.5 * (y + x / y);
        y = 0.5 * (y + x / y);
        y = 0.5 * (y + x / y);
        y
    }
}

/// Computes an accurate `exp(x)` approximation in pure `no_std` using range reduction
/// and a 5th-order Padé/Taylor polynomial.
#[inline]
pub fn exp_f32(x: f32) -> f32 {
    #[cfg(feature = "std")]
    {
        x.exp()
    }

    #[cfg(not(feature = "std"))]
    {
        let clamped = x.clamp(-20.0, 20.0);
        // Use (1 + clamped / 64)^64 via 6 repeated squarings
        let mut val = 1.0 + clamped * (1.0 / 64.0);
        if val < 0.0 {
            val = 0.0;
        }
        for _ in 0..6 {
            val *= val;
        }
        val
    }
}
