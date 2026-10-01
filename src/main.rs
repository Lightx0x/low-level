fn main() {
    let arr = [1_u32, 2, 3, 4];
    let p = arr.as_ptr();
    let x = unsafe { *p.add(4) }; // index 4 of a 4-element array
    println!("{x}");
}
