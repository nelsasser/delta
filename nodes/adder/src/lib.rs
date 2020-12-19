pub fn add_integers(x: i32, y: i32) -> i32 {
    x + y
}

#[cfg(test)]
mod tests {
    use crate::add_integers;
    #[test]
    fn it_works() {
        assert_eq!(add_integers(2, 2), 4);
    }
}
