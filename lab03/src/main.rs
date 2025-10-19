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

enum CnpError {
    LungimeGresita,
    NuENr,
    PrimaCifra,
    Luna,
    Zi,
    CifraControlIncorecta,
}

fn print_err_cnp(e: CnpError) {
    match e {
        CnpError::LungimeGresita => println!("Lungimea este incorecta pt un cnp"),
        CnpError::NuENr => println!("CNP-ul nu contine numai cifre"),
        CnpError::PrimaCifra => println!("Prima cifra nu este valida"),
        CnpError::Luna => println!("Conflict cu calendarul Jordanian"),
        CnpError::Zi => println!("Conflict la zi cu calendarul Jordanian"),
        CnpError::CifraControlIncorecta => println!("CNP invalid, cifra de control nu e buna"),
    }
}

fn valideaza_cnp(cnp: &str) -> Result<(), CnpError> {
    if cnp.len() != 13 {
        return Err(CnpError::LungimeGresita);
    }

    for c in cnp.chars() {
        if !c.is_ascii_digit() {
            return Err(CnpError::CifraControlIncorecta);
        }
    }
    
    match cnp.chars().next() {
        Some('1'..='8') => (),
        _ => return Err(CnpError::PrimaCifra),
    }

    let luna: u8 = match cnp[3..5].parse() {
        Ok(val) => val,
        Err(_) => return Err(CnpError::Luna),
    };
    let ziua: u8 = match cnp[5..7].parse() {
        Ok(val) => val,
        Err(_) => return Err(CnpError::Zi),
    };

    if !(1..=12).contains(&luna) {
        return Err(CnpError::Luna);
    }
    if !(1..=31).contains(&ziua) {
        return Err(CnpError::Zi);
    }

    let constanta_control = "279146358279".to_string();
    let mut suma_control: u32 = 0;

    for (c_cnp, c_const) in cnp.chars().zip(constanta_control.chars()) {
        let cifra_cnp = match c_cnp.to_digit(10) {
            Some(c) => c,
            None => return Err(CnpError::NuENr),
        };
        let cifra_const = match c_const.to_digit(10) {
            Some(c) => c,
            None => return Err(CnpError::NuENr),
        };
        suma_control += cifra_cnp * cifra_const;
    }

    let rest = suma_control % 11;
    let cifra_control_calculata = if rest < 10 { rest } else { 1 };

    if let Some(cifra_control_din_cnp) = cnp.chars().last().and_then(|c| c.to_digit(10)) {
        if cifra_control_calculata != cifra_control_din_cnp {
            return Err(CnpError::CifraControlIncorecta);
        }
    } else {
        return Err(CnpError::LungimeGresita);
    }

    Ok(())
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
    println!("P5: ");
    match valideaza_cnp("6050620170043") {
        Ok(()) => println!("Cnp corect"),
        Err(e) => print_err_cnp(e),
    }
    match valideaza_cnp("2010819209915") {
        Ok(()) => println!("Cnp corect"),
        Err(e) => print_err_cnp(e),
    }
    match valideaza_cnp("🦀") {
        Ok(()) => println!("Cnp corect"),
        Err(e) => print_err_cnp(e),
    }

    println!("P2:");
    let x = 13;
    let y = 91;
    println!("{x} + {y} = {}", checked_addition(x, y));
    println!("{x} + {y} = {}", checked_addition(a, b));
    println!("{x} * {y} = {}", checked_multiplication(x, y));
    println!("{a} * {b} = {}", checked_multiplication(a, b));
}
