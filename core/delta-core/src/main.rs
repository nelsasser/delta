use delta_node::DeltaNode;
use arithmetic::Addi32;

fn main() {
    println!("Hello, world!");
    let mut adder: Box<Addi32> = Addi32::initialize();
    adder.set_x(2);
    adder.set_y(2);
    println!("2 plus 2 is {}", adder.execute());
}
