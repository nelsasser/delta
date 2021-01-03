pub trait DeltaNode<T> {
    fn execute(self) -> T;
}