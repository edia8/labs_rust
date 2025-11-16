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

use core::fmt;
use std::fmt::Debug;
use std::ops::{Add,Sub,Mul,Neg,AddAssign,SubAssign,MulAssign};
use std::cmp::PartialEq;
#[derive(Debug,Clone, Copy)]
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
impl Default for Complex {
    fn default() -> Self {
        Complex { real: 0f64, imag: 0f64 }
    }
}
impl From<i32> for Complex {
    fn from(value: i32) -> Self {
        Complex { real: value as f64, imag: 0f64 }
    }
}
impl From<f64> for Complex {
    fn from(value: f64) -> Self {
        Complex { real: value, imag: 0f64 }
    }
}

impl<T> Add<T> for Complex 
where 
    Complex:From<T>{
    type Output = Complex;
    fn add(self, param: T) -> Self::Output {
        let param = Complex::from(param);
        Self {
            real : self.real + param.real,
            imag : self.imag + param.imag,
        }
    }
}
impl<T> Sub<T> for Complex 
where 
    Complex:From<T>{
    type Output = Self;
    fn sub(self, param: T) -> Self::Output {
        let param = Complex::from(param);
        Self {
            real : self.real - param.real,
            imag : self.imag - param.imag,
        }
    }
}
impl<T> Mul<T> for Complex 
where
    Complex:From<T>{
    type Output = Self;
    fn mul(self, param: T) -> Self::Output {
        let param = Complex::from(param);
        Self {
            real : self.real * param.real - self.imag * param.imag,
            imag : self.real * param.imag + self.imag * param.real,
        }
    }
}
impl Neg for Complex {
    type Output = Complex;
    fn neg(self) -> Self::Output {
        Complex{real:-self.real,imag:-self.imag}
    }
}
impl<T> AddAssign<T> for Complex
where
    Complex:From<T> {
    fn add_assign(&mut self, rhs: T) {
        let rhs = Complex::from(rhs);
        self.real += rhs.real;
        self.imag += rhs.imag;
    }
}
impl<T> SubAssign<T> for Complex
where
    Complex:From<T> {
        fn sub_assign(&mut self, rhs: T) {
            let rhs = Complex::from(rhs);
            self.real -= rhs.real;
            self.imag -= rhs.imag;
        }
    }
impl<T> MulAssign<T> for Complex
where
    Complex:From<T> {
        fn mul_assign(&mut self, rhs: T) {
            let rhs = Complex::from(rhs);
            let rnew = self.real * rhs.real - self.imag * rhs.imag;
            let inew = self.real * rhs.imag + self.imag * rhs.real;
            *self = Complex::new(rnew, inew);
        }
    }
impl PartialEq for Complex {
    fn eq(&self, other: &Self) -> bool {
        self.real == other.real && self.imag == other.imag
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl fmt::Display for Complex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Complex{real:0f64,imag:0f64} => write!(f,"0"),
            Complex{real,imag:0f64} => f.write_fmt(format_args!("{}",*real)),
            Complex{real:0f64,imag} => f.write_fmt(format_args!("{}i",*imag)),
            Complex{real,imag} if self.imag > 0f64 => f.write_fmt(format_args!("{}+{}i",*real,*imag)),
            Complex{real,imag} => f.write_fmt(format_args!("{}{}i",*real,*imag))
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

    // Note: .to_string() uses Display to format the type
    assert_eq!(Complex::new(1, 2).to_string(), "1+2i");
    assert_eq!(Complex::new(1, -2).to_string(), "1-2i");
    assert_eq!(Complex::new(0, 5).to_string(), "5i");
    assert_eq!(Complex::new(7, 0).to_string(), "7");
    assert_eq!(Complex::new(0, 0).to_string(), "0");

    let h = Complex::new(-4, -5);
    let i = h - (h + 5) * 2.0;
    assert_eq_rel!(i.real, -6);

    let j = -i + i;
    assert_eq_rel!(j.real, 0);
    assert_eq_rel!(j.imag, 0);
    //BONUS
    let mut k = Complex::default();
    k+=h;
    assert_eq_rel!(k.real,-4);
    assert_eq_rel!(k.imag,-5);
    k*=a;
    assert_eq_rel!(k.real,6);
    assert_eq_rel!(k.imag,-13);
    k-=k;
    assert_eq_rel!(k.real,0);
    assert_eq_rel!(k.imag,0);
    println!("ok!");
}