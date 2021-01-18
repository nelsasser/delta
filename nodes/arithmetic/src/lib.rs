use delta_lib_macro::{delta_node_struct, delta_node_impl, RegisterDeltaNode};
use delta_lib::DeltaNode;
#[delta_node_struct]
#[derive(RegisterDeltaNode)]
pub struct Addi32 {
    // potential flag ideas for fields:
    // delta-ignore:    - Tells macros to ignore this variable when generating set and reset functions.
    //                  - Also can be used on public fields so that they are included. Public fields would be excluded by default (or maybe have own flag?).
    //
    // delta-noreset:   - Tells macros not to generate reset code for the field.
    //                  - Can be used to retain state throughout executions and only changes when new data comes in.
    //
    // delta-default => {...}:   - Specify the default value of the field when reseting. 
    //                           - The data inside the `{}` will be taken as gospel, so it must be 100% syntactically correct.
    
    x: i32,
    y: i32,
}

#[delta_node_impl]
impl Addi32 {
    // these will be generated (by the struct macro )

    // maybe to avoid the ugly double underscore we can add an attribute flag to the methods to signal the macro
    // macro would then wrap the method in the standardized name 
    // if an attribute flag doesn't exist and the double underscore name doesn't exist then generate a default method


    // idea for the initialization process.
    // Split into two stages, the default initialization and custom intialization
    // 1) Default Initialization. 
    //  - It is always generated and run and uses the default values from the struct, either specified by the user or derived by the type
    //  - Static method, returns a Box<T> where T is the node type
    //
    // 2) Custom Initialization.
    //  - User specified initialization values.
    //  - If it is registered then it runs after the default initialization and overrites the current values
    //  - Takes in a mutable reference to itself, so user picks which values to initialize
    //  - Only modifying itself, so it returns nothing
    //
    // Then, after both intitialization functions have run, the Box from the default stage carrying the changes of the custom stage is returned.
    // pub fn __initialize() -> Box<Addi32> {
    //     // create a new blank Addi32 node on the heap
    //     Box::new(Addi32 { x: 0, y: 0, __num_attributes: 0, __set_attributes: 0}) // what it will look like (hopefully the error will go away because it will see that it expands correctly)
    //     // Box::new(Addi32 { x: 0, y: 0, __num_attributes: 2, __set_attributes: 0}) // what it will be generated
    // }

    // fn __pre_execute(&mut self) {
    //     // do nothing before execution
    //     println!("Pre Execution!!!");
    // }

    // fn __on_execute(&mut self) -> i32 {
    //     println!("Execution");
    //     self.x + self.y
    // }

    // fn __post_execute(&mut self) {
    //     // reset both values after executing node
    //     self.x = 0;
    //     self.y = 0;
    //     println!("Post Execution");
    // }
}

#[cfg(test)]
mod tests {
    use crate::DeltaNode;
    use crate::Addi32;

    #[test]
    fn it_works() {
        let mut adder: Box<Addi32> = Addi32::__initialize();
        adder.__set_x(100);
        adder.__set_y(100);
        assert_eq!(200, adder.__execute());
    }
}