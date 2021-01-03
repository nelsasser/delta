use delta_node_macro::{delta_node_attributes, RegisterDeltaNode};
use delta_node::DeltaNode;
#[delta_node_attributes]
#[derive(RegisterDeltaNode)]
pub struct Addi32 {
    x: i32,
    y: i32,
}

impl Addi32 {
    pub fn set_x(&mut self, x: i32) { self.x = x; }
    pub fn set_y(&mut self, y: i32) { self.y = y; }

    pub fn initialize() -> Box<Addi32> {
        // create a new blank Addi32 node on the heap
        Box::new(Addi32 { x: 0, y: 0, __num_attributes: 2, __set_attributes: 0})
    }

    fn pre_execute(&mut self) {
        // do nothing before execution
        println!("Pre Execution!!!");
    }

    fn on_execute(&mut self) -> i32 {
        println!("Execution");
        self.x + self.y
    }

    fn post_execute(&mut self) {
        // reset both values after executing node
        self.x = 0;
        self.y = 0;
        println!("Post Execution");
    }
}

#[cfg(test)]
mod tests {
    use crate::DeltaNode;
    use crate::Addi32;

    #[test]
    fn it_works() {
        let mut adder: Box<Addi32> = Addi32::initialize();
        adder.set_x(100);
        adder.set_y(100);
        assert_eq!(200, adder.execute());
    }
}