fn is_prime(x: u16) -> bool {
    if x < 2 {
        return false;
    }
    if x % 2 == 0 && x != 2 {
        return false;
    }
    let mut i: u16 = 3;
    while i <= x / i {
        if x % i == 0 {
            return false;
        }
        i += 2
    }
    true
}

fn next_prime(x: u16) -> Option<u16> {
    let mut z = x.checked_add(1)?;
    loop {
        if is_prime(z) {
            return Some(z);
        }
        z = z.checked_add(1)?;
    }
}

fn checked_addition(x: u32, y: u32) -> u32 {
    if u32::MAX - x < y {
        panic!("The value {x}+{y} does not fit in a u32 variable");
    }
    x + y
}

fn checked_multiplication(x: u32, y: u32) -> u32 {
    if x == 0 || y == 0 {
        return 0;
    } else if u32::MAX / x < y {
        panic!("The value {x}*{y} does not fit in a u32 variable");
    }
    x * y
}

// 3
#[derive(Debug)]
enum Err {
    OverflowAddition,
    OverflowMultiplication,
}

fn result_checked_adition(x: u32, y: u32) -> Result<u32, Err> {
    if u32::MAX - x < y {
        return Err(Err::OverflowAddition);
    }
    Ok(x + y)
}

fn result_checked_multiplication(x: u32, y: u32) -> Result<u32, Err> {
    if x == 0 || y == 0 {
        return Ok(0);
    } else if u32::MAX / x < y {
        return Err(Err::OverflowMultiplication);
    }
    Ok(x * y)
}

fn use_function_add(x: u32, y: u32) -> Result<u32, Err> {
    Ok(result_checked_adition(x, y))?
}
fn use_function_multiply(x: u32, y: u32) -> Result<u32, Err> {
    Ok(result_checked_multiplication(x, y))?
}

//4

#[derive(Debug)]
enum Erori {
    NotASCII(char),
    NotDigit(char),
    NotBase16(char),
    NotLetter(char),
    NotPrintable(char),
}

fn to_uppercase(x: char) -> Result<char, Erori> {
    if x.is_alphabetic() {
        Ok(x.to_ascii_uppercase())
    } else {
        Err(Erori::NotLetter(x))
    }
}

fn to_lowercase(x: char) -> Result<char, Erori> {
    if x.is_alphabetic() {
        Ok(x.to_ascii_lowercase())
    } else {
        Err(Erori::NotLetter(x))
    }
}

fn print_char(x: char) -> Result<char, Erori> {
    if x.is_ascii_graphic() || x.is_whitespace() {
        Ok(x)
    } else {
        Err(Erori::NotPrintable(x))
    }
}

fn char_to_number(x: char) -> Result<u8, Erori> {
    if !x.is_ascii() {
        return Err(Erori::NotASCII(x));
    }

    if x.is_ascii_digit() {
        Ok(x as u8 - b'0')
    } else {
        Err(Erori::NotDigit(x))
    }
}

fn char_to_number_hex(x: char) -> Result<u8, Erori> {
    if !x.is_ascii() {
        return Err(Erori::NotASCII(x));
    }

    match x {
        '0'..='9' => Ok(x as u8 - b'0'),
        'a'..='f' => Ok(x as u8 - b'a' + 10),
        'A'..='F' => Ok(x as u8 - b'A' + 10),
        _ => Err(Erori::NotBase16(x)),
    }
}

fn print_error(e: Erori) {
    match e {
        Erori::NotASCII(x) => println!("{x} nu este caracter ASCII"),
        Erori::NotBase16(x) => println!("{x} nu este cifra hexazecimala"),
        Erori::NotDigit(x) => println!("{x} nu este cirfa"),
        Erori::NotLetter(x) => println!("{x} nu este litera"),
        Erori::NotPrintable(x) => println!("{x} nu poate fi printat"),
    }
}
fn main() {
    println!("P1:");
    let mut x = 65371u16;
    while let Some(p) = next_prime(x) {
        println!("{p}");
        x = p;
    }

    let a: u32 = 4000000000;
    let b: u32 = 4000000000;
    println!("P3: ");
    match use_function_add(a, b) {
        Ok(val) => println!("{val}"),
        Err(e) => println!("{e:?}"),
    }
    match use_function_multiply(a, b) {
        Ok(val) => println!("{val}"),
        Err(e) => println!("{e:?}"),
    }
    println!("P4: ");
    let s = String::from("Ab 12🦀");

    println!("\nto_lowercase:");
    for c in s.chars() {
        match to_lowercase(c) {
            Ok(val) => println!("{val}"),
            Err(e) => print_error(e),
        }
    }
    println!("\nto_uppercase:");
    for c in s.chars() {
        match to_uppercase(c) {
            Ok(val) => println!("{val}"),
            Err(e) => print_error(e),
        }
    }
    println!("\nprint_char:");
    for c in s.chars() {
        match print_char(c) {
            Ok(val) => println!("{val}"),
            Err(e) => print_error(e),
        }
    }
    println!("\nchar_to_number:");
    for c in s.chars() {
        match char_to_number(c) {
            Ok(val) => println!("{val}"),
            Err(e) => print_error(e),
        }
    }
    println!("\nchar_to_number_hex:");
    for c in s.chars() {
        match char_to_number_hex(c) {
            Ok(val) => println!("{val}"),
            Err(e) => print_error(e),
        }
    }
    println!("P2:");
    let x = 13;
    let y = 91;
    println!("{x} + {y} = {}", checked_addition(x, y));
    println!("{x} + {y} = {}", checked_addition(a, b));
    println!("{x} * {y} = {}", checked_multiplication(x, y));
    println!("{a} * {b} = {}", checked_multiplication(a, b));
}
