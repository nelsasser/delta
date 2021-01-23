// is this even necessary??? I am leaning towards no...
// but it does provide a standard interface and generics where required, so I guess it works?
pub trait DeltaNode<T> {
    fn __execute(self) -> T;
    fn __initialize() -> Box<T>;
}


pub enum DeltaImpulse<T> {
    NOOP, //no op
    SEND(T), // pass message
    TICK, // 
    //LOG(DeltaMessage), //
}