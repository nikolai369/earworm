use std::ops::{Add, AddAssign, Mul};

pub struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    pub fn new(real: f64, im: f64) -> Self {
        Complex { re: real, im }
    }

    // r is the magnitude of the Complex num and theta is the angle of the vector
    pub fn from_polar(r: f64, theta: f64) -> Self {
        Complex {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    pub fn abs(&self) -> f64 {
        (self.im.powi(2) + self.re.powi(2)).sqrt()
    }

    pub fn powi(&self, n: i32) -> Self {
        let r = self.abs().powi(n); // calculate the magnitude and raise to n
        let theta = self.im.atan2(self.re) * n as f64; // calculate the angle and multiply it by n
        Complex::from_polar(r, theta)
    }
}

impl Add for Complex {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Complex {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl AddAssign for Complex {
    fn add_assign(&mut self, rhs: Self) -> () {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

// (a + bi) * (c + di)
// ac + adi + cbi + bdi^2
// ac + adi + cbi - bd
// (ac - bd) + (ad + cb)i
impl Mul for Complex {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Complex {
            re: (self.re * rhs.re) - (self.im * rhs.im),
            im: (self.re * rhs.im) + (self.im * rhs.re),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let c = Complex::new(3.0, 4.0);
        assert_eq!(c.re, 3.0);
        assert_eq!(c.im, 4.0);
    }

    #[test]
    fn test_add() {
        let a = Complex::new(1.0, 2.0);
        let b = Complex::new(3.0, 4.0);
        let result = a + b;
        assert_eq!(result.re, 4.0);
        assert_eq!(result.im, 6.0);
    }

    #[test]
    fn test_add_assign() {
        let mut a = Complex::new(1.0, 1.0);
        let b = Complex::new(2.0, 3.0);
        a += b;
        assert_eq!(a.re, 3.0);
        assert_eq!(a.im, 4.0);
    }

    #[test]
    fn test_abs() {
        let c = Complex::new(3.0, 4.0);
        let abs = c.abs();
        assert!((abs - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_mul() {
        let a = Complex::new(1.0, 2.0);
        let b = Complex::new(3.0, 4.0);
        let result = a * b;
        // (1*3 - 2*4, 1*4 + 2*3) = (-5, 10)
        assert!((result.re + 5.0).abs() < 1e-10);
        assert!((result.im - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_from_polar() {
        let r = 1.0;
        let theta = std::f64::consts::PI / 2.0;
        let c = Complex::from_polar(r, theta);
        assert!((c.re).abs() < 1e-10); // cos(π/2) ≈ 0
        assert!((c.im - 1.0).abs() < 1e-10); // sin(π/2) ≈ 1
    }

    #[test]
    fn test_powi() {
        let c = Complex::new(1.0, 1.0);
        let result = c.powi(2);
        // (1 + i)^2 = 1 + 2i + i^2 = (1 - 1) + 2i = 2i
        assert!((result.re).abs() < 1e-10);
        assert!((result.im - 2.0).abs() < 1e-10);
    }
}
