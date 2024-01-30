pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[no_mangle]
pub extern "system" fn enable_hook(ptr: usize) -> usize {
    0
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
