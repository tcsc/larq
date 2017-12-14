/**
 * Wraps up access to a backup repository
 */
pub struct Repository {}

impl Repository {
    pub fn new() -> Repository {
        Repository {}
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
