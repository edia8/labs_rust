//P1

fn add_chars_n(mut s: String, c: char, x: u8) -> String {
    for _ in 0..x {
        s.push(c);
    }
    s
}
fn p1() {
    let mut s = String::from("");
    let mut i = 0;
    while i < 26 {
        let c = (i + b'a') as char;
        s = add_chars_n(s, c, 26 - i);

        i += 1;
    }

    println!("P1:\n{s}");
}

//P2

fn add_chars_n_void(s: &mut String, c: char, x: u8) {
    for _ in 0..x {
        s.push(c);
    }
}

fn p2() {
    let mut s = String::from("");
    let ref_s: &mut String = &mut s;
    let mut i = 0;
    while i < 26 {
        let c = (i + b'a') as char;
        add_chars_n_void(ref_s, c, 26 - i);

        i += 1;
    }
    println!("P2:\n{s}");
}

//P3

fn add_space(s: &mut String, x: u8) {
    for _ in 0..x {
        s.push(' ');
    }
}
fn add_str(s: &mut String, a: String) {
    *s += &a;
}

fn add_integer(s: &mut String, number: u32) {
    let nr: f32 = number as f32;
    let len = f32::log10(nr);
    let len_32: u8 = len as u8 + 1u8;
    let mut x = 1u32;
    for _ in 1..len_32 {
        x *= 10;
    }

    for i in 1..=len_32 {
        let car = (((number / x) % 10u32) as u8 + b'0') as char;
        s.push(car);
        if i % 3 == 0 && i != len_32 {
            s.push('_')
        }
        x /= 10;
    }
}

fn add_float(s: &mut String, number: f32) {
    let integer = number as u32;
    let mut fractionar = number - number.trunc();
    add_integer(s, integer);
    add_str(s, String::from("."));
    let mut cnt = 0;
    loop {
        if fractionar != 0f32 && cnt >= 3 {
            break;
        }
        let digit = ((fractionar * 10f32).trunc()) as u8;
        fractionar = fractionar * 10f32 - (fractionar * 10f32).trunc();
        s.push((digit + b'0') as char);
        cnt += 1;
    }
}

fn p3() {
    let mut s = String::from("");
    add_space(&mut s, 40);
    add_str(&mut s, String::from("I 💚\n"));
    add_space(&mut s, 40);
    add_str(&mut s, String::from("RUST.\n"));
    add_space(&mut s, 4);
    add_str(&mut s, String::from("Most"));
    add_space(&mut s, 12);
    add_str(&mut s, String::from("crate"));
    add_space(&mut s, 6);
    add_integer(&mut s, 306437968);
    add_space(&mut s, 11);
    add_str(&mut s, String::from("and"));
    add_space(&mut s, 5);
    add_str(&mut s, String::from("lastest"));
    add_space(&mut s, 9);
    add_str(&mut s, String::from("is\n"));
    add_space(&mut s, 9);
    add_str(&mut s, String::from("downloaded"));
    add_space(&mut s, 8);
    add_str(&mut s, String::from("has"));
    add_space(&mut s, 13);
    add_str(&mut s, String::from("downloads"));
    add_space(&mut s, 5);
    add_str(&mut s, String::from("the"));
    add_space(&mut s, 9);
    add_str(&mut s, String::from("version"));
    add_space(&mut s, 4);
    add_float(&mut s, 2.038);
    add_str(&mut s, String::from("."));
    println!("P3:\n{s}");
}

fn main() {
    p1();
    p2();
    p3();
}
