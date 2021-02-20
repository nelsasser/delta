use delta_lib_macro::{delta_node_struct, delta_node_impl, RegisterDeltaNode};
use delta_lib::{DeltaNode, Impulse};

#[delta_node_struct]
#[derive(RegisterDeltaNode)]
pub struct Addi32 {
    // potential flag ideas for fields:
    // delta-ignore:    - Tells macros to ignore this variable when generating set and reset functions.
    //                  - Also can be used on public fields so that they are included. Public fields would be excluded by default
    //                  - Can be set either with or without arguments [true, false], by default true will be used
    //                  - Setting equal to false can allow functions to be generated for public variables 
    //
    // delta-noreset:   - Tells macros not to generate reset code for the field.
    //                  - Can be used to retain state throughout executions and only changes when new data comes in.
    //                      
    // delta-default(...):  - Specify the default value of the field when reseting. 
    //                      - The data inside the `()` will be taken as gospel, so it must be 100% syntactically correct.
    x: i32,
    y: i32,

    #[delta_ignore] // by default, if no arguments are specified then the delta_ignore will be true
    #[delta_default(10)] // we can have constants
    /// # comments! (we don't want these yuck!)
    my_ignored: i32,// indicate to: 1) Do not generate any set or reset functions for the field
                    //              2) Set the default value upon initialization to 10.
                    // These attributes effectively makes it so that the engine has no influence on this variable after the node is initialized.

    #[delta_noreset]
    #[delta_default("Hello World!")]
    custom_reset: String, // indicate that we want to generate set and reset functions, but that we don't want the reset function to be called automatically

    pub my_default_public: f32, // this will not have anything generate for it

    #[delta_ignore(false)]
    //#[delta_default(match query_db() { Ok(x: f32) => { x }, Err(_) => { 0.0_f32 }, })] // or even small bits of code
    pub my_generated_public: f32, // this will have set and reset functions generated, and will be included in each reset

    #[delta_ignore(false)]
    #[delta_noreset]
    pub my_controlled_public: i64, // this will generate set and reset functions but will not be included in the overall reset
}

// TODO: allow user to choose to append or prepend their custom post_execute to before or after the default reset if they don't want to fully override it
#[delta_node_impl(on_exec = "custom_execute", post_exec = "custom_postexecute", one_off)]
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
    //     Box::new(Addi32 { x: 0, y: 0}) // what it will look like (hopefully the error will go away because it will see that it expands correctly)
    //     Box::new(Addi32 { x: 0, y: 0, __num_attributes: 2, __set_attributes: 0}) // what it will be generated
    // }


    // the macro will recognize that __pre_execute is already implemented and use it
    fn __pre_execute(&mut self) {
        // do nothing before execution
        println!("Pre Execution!!!");
    }

    // we can also name the functions whatever we want,
    // then we can signal to the macro to use the custom function as whatever 
    fn custom_execute(&mut self) -> Impulse<i32> {
        let result = self.x + self.y;
        self.my_ignored += 1;
        Impulse::SEND(result)
    }

    fn custom_postexecute(&mut self) {
        println!("I have added {} times so far!", self.my_ignored);
    }
}


#[cfg(test)]
mod tests {
    use crate::DeltaNode;
    use crate::Impulse;
    use crate::Addi32;

    #[test]
    fn it_works() {
        let mut adder: Box<Addi32> = Addi32::__initialize();
        adder.__set_x(100);
        adder.__set_y(100);
        assert_eq!(Impulse::SEND(200), adder.__execute());
    }
}