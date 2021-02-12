// is this even necessary??? I am leaning towards no...
// but it does provide a standard interface and generics where required, so I guess it works?
pub trait DeltaNode<ReturnType, NodeType> { // is there a better naming convention?
    fn __execute(self) -> ReturnType;
    fn __initialize() -> Box<NodeType>;
}

#[derive(Debug, PartialEq)]
pub enum Impulse<T> {
    NOOP, //no op
    SEND(T), // pass message
    TICK, // step tell engine to step forward
    //LOG(DeltaMessage), //
}

#[cfg(test)]
mod tests {
    use crate::Impulse;
    #[test]
    fn it_works() {
        let noop: Impulse<i32> = Impulse::NOOP;
        assert_eq!(noop, Impulse::<i32>::NOOP);

        let num_send: Impulse<i32> = Impulse::SEND(100);
        assert_eq!(num_send, Impulse::<i32>::SEND(100));

        let my_vec: Vec<f32> = vec![100.0, 1.0, 0.0, 20.0];
        let vec_send: Impulse<&Vec<f32>> = Impulse::SEND(&my_vec);
        assert_eq!(vec_send, Impulse::<&Vec<f32>>::SEND(&my_vec));
    }
}