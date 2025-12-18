use core::str;
use std::{fs, io, process::exit};

fn read_from_file(str: &str) -> Result<String, io::Error> {
    let informatie: String = fs::read_to_string(str)?;
    Ok(informatie)
}

fn p1() {
    let path = "input.txt";
    let info = read_from_file(path);
    let mut m = String::from("");
    match info {
        Ok(vaule) => m.push_str(&vaule),
        Err(e) => println!("Eroare la citire fisier: {e}"),
    }
    let mut str_bytes = String::from("");
    let mut len_str_bytes: u32 = 0;
    let mut str_chars = String::from("");
    let mut len_str_chars: u32 = 0;
    for s in m.lines() {
        if s.len() as u32 > len_str_bytes {
            len_str_bytes = s.len() as u32;
            str_bytes = s.to_string();
        }
        let mut counter: u32 = 0;
        for _ in s.chars() {
            counter += 1;
        }
        if len_str_chars < counter {
            len_str_chars = counter;
            str_chars = s.to_string();
        }
    }
    println!(
        "P1:\nCea mai lunga linie in bytes: {str_bytes}\nCea mai lunga linie in caractere: {str_chars}"
    );
}

fn p2(path: &'static str) {
    //let path = "input_2.txt";
    let info = read_from_file(path);
    let mut sir = String::from("");
    let mut sir_nou = String::from("");
    match info {
        Ok(val) => sir.push_str(&val),
        Err(e) => println!("Eroare la citire fisier: {e}"),
    }
    if !sir.is_ascii() {
        println!("Sirul nu are numai ascii");
        exit(2);
    }

    for c in sir.chars() {
        if c.is_ascii_lowercase() {
            let d = b'a' + (c as u8 - b'a' + 13u8) % 26u8;
            sir_nou.push(d as char);
        } else if c.is_ascii_uppercase() {
            let d = b'A' + (c as u8 - b'A' + 13u8) % 26u8;
            sir_nou.push(d as char);
        } else {
            sir_nou.push(c);
        }
    }
    println!("P2:\n{sir_nou}");
}

fn p3() {
    let path = "prescurtari.txt";
    let info = read_from_file(path);
    let info2 = read_from_file("prop.txt");
    let mut inlocuiri = String::from("");
    let mut prop = String::from("");
    match info {
        Ok(val) => inlocuiri.push_str(&val),
        Err(e) => println!("Eroare este {e}"),
    }
    match info2 {
        Ok(val) => prop.push_str(&val),
        Err(e) => println!("Eroarea este {e}"),
    }
    println!("P3:\n{prop}");
    let mut it = inlocuiri.rsplit([' ', '\n']);
    while let Some(l) = it.next() {
        prop = prop.replace(l, it.next().unwrap());
    }
    println!("{prop}");
}

fn p4() {
    println!("P4:");
    let path = "/etc/hosts";
    let info = read_from_file(path);
    let mut hosts = String::from("");
    match info {
        Ok(val) => hosts.push_str(&val),
        Err(e) => println!("Eroarea este {e}"),
    }
    for str in hosts.lines() {
        if !str.starts_with('#')
            && let Some((str1, mut str2)) = str.split_once([' ', '\t'])
        {
            str2 = str2.trim();
            if let Some((str3, _str4)) = str2.split_once(' ') {
                str2 = str3
            }
            println!("{str2} => {str1}");
        }
    }
}
// fn bonus() {
//     //p2("bonus.txt");
//     //time: 149.231818188s
//     let s = String::from("");
//     let mut buf = [0u8;4096];

// }

fn main() {
    p1();
    p2("input2.txt");
    p3();
    p4();
    //let start = Instant::now();
    // bonus();
    //println!("{:?}", start.elapsed());
}
