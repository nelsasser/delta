// is this even necessary??? I am leaning towards no...
pub trait DeltaNode<T> {
    fn __execute(self) -> T;
}