fn eq_rel(x: f64, y: f64) -> bool {
    (x - y).abs() < 0.001
}
// This is a macro that panics if 2 floats are not equal using an epsilon.
// You are not required to understand it yet, just to use it.
macro_rules! assert_eq_rel {
    ($x:expr, $y: expr) => {
        let x = $x as f64;
        let y = $y as f64;
        let r = eq_rel(x, y);
        assert!(r, "{} != {}", x, y);
    };
}

use std::fmt::Debug;
use std::ops::{Add,Sub,Mul};
use std::cmp::PartialEq;
#[derive(Clone, Copy)]
struct Complex {
    real:f64,
    imag:f64,
}

impl Complex {
    fn new<T,K>(a:T,b:K) -> Complex
    where 
    f64: From<T>,
    f64: From<K>,
     {
        Self { real: a.into(), imag: b.into() }
    }
    fn conjugate(&self) ->Complex{
        Self { real: self.real, imag: -self.imag }
    }
}
impl Add for Complex {
    type Output = Self;
    fn add(self, param: Self) -> Self::Output {
        Self {
            real : self.real + param.real,
            imag : self.imag + param.imag,
        }
    }
}
impl Sub for Complex {
    type Output = Self;
    fn sub(self, param: Self) -> Self::Output {
        Self {
            real : self.real - param.real,
            imag : self.imag - param.imag,
        }
    }
}
impl Mul for Complex {
    type Output = Self;
    fn mul(self, param: Self) -> Self::Output {
        Self {
            real : self.real * param.real - self.imag * param.imag,
            imag : self.real * param.imag + self.imag * param.real,
        }
    }
}
impl PartialEq for Complex {
    fn eq(&self, other: &Self) -> bool {
        self.real == other.real && self.imag == other.imag
    }
}
impl Debug for Complex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.imag > 0f64 {
            f.write_fmt(format_args!("{}+{}i",self.real,self.imag))
        } else {
            f.write_fmt(format_args!("{}{}i",self.real,self.imag))
        }
    }
}

fn main() {
    let a = Complex::new(1.0, 2.0);
    assert_eq_rel!(a.real, 1);
    assert_eq_rel!(a.imag, 2);

    let b = Complex::new(2.0, 3);
    let c = a + b;
    assert_eq_rel!(c.real, 3);
    assert_eq_rel!(c.imag, 5);

    let d = c - a;
    assert_eq!(b, d);

    let e = (a * d).conjugate();
    assert_eq_rel!(e.imag, -7);

    let f = (a + b - d) * c;
    assert_eq!(f, Complex::new(-7, 11));
    println!("ok");
}