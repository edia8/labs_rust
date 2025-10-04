fn gcd(mut a: u8, mut b: u8) -> u8 {
    if a == 0 || b == 0 {
        return 0;
    }
    let cmmdc = loop {
        if a > b {
            a -= b
        } else if b > a {
            b -= a
        } else {
            break a;
        }
    };
    cmmdc
}

fn is_prime(x: u8) -> bool {
    if x < 2 {
        return false;
    }
    if x % 2 == 0 && x != 2 {
        return false;
    }
    let mut i: u8 = 3;
    while i <= x / i {
        if x % i == 0 {
            return false;
        }
        i += 2
    }
    true
}

fn main() {
    for i in 0u8..=100u8 {
        println!("{} : {}", i, is_prime(i))
    }
    for i in 0u8..100u8 {
        for j in i + 1..=100u8 {
            println!("({},{}) = {}", i, j, gcd(i, j))
        }
    }
    let mut x = 99u8;
    loop {
        println!(
            "{} bottles of beer on the wall,\n{} bottles of beer.\nTake one down, pass it around,",
            x, x
        );
        x -= 1;
        if x == 0 {
            break;
        }
        println!("{} bottles of beer on the wall.\n\n", x);
    }
    println!("No bottle of beer on the wall\n")
}
