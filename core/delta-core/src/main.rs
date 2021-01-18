use delta_lib::DeltaNode;
use arithmetic::Addi32;

fn main() {
    println!("Hello, world!");
    let mut adder: Box<Addi32> = Addi32::__initialize();
    adder.__set_x(2);
    adder.__set_y(2);
    println!("2 plus 2 is {}", adder.__execute());
}
