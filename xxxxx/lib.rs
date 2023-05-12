pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub mod apps;

pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
