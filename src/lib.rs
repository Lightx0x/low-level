pub fn hexa() {
    let n = 32767;
    println!("{n:b}"); // binary
    println!("{n:x}"); // hex
    println!("{n:#x}"); // hex with the 0x label
    let m = 0xFF; // you can write numbers in hex too
    println!("{m}"); // prints in decimal
}

#[cfg(test)]
mod test {
    // use super::*;

    // #[test]
    // fn hex_values() {
    //
    // }
}
