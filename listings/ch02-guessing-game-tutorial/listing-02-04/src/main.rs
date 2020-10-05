// ANCHOR: here
use rand::Rng;
use std::cmp::Ordering;
use std::io;

fn main() {
    // --snip--
    // ANCHOR_END: here
    println!("請猜測一個數字！");

    let secret_number = rand::thread_rng().gen_range(1, 101);

    println!("祕密數字爲：{}", secret_number);

    println!("請輸入你的猜測數字。");

    let mut guess = String::new();

    io::stdin()
        .read_line(&mut guess)
        .expect("讀取行數失敗");
    // ANCHOR: here

    println!("你的猜測數字：{}", guess);

    match guess.cmp(&secret_number) {
        Ordering::Less => println!("太小了！"),
        Ordering::Greater => println!("太大了！"),
        Ordering::Equal => println!("獲勝！"),
    }
}
// ANCHOR_END: here
