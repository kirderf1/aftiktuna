pub struct Parse<'a> {
    input: &'a str,
}

impl<'a> Parse<'a> {
    pub fn new(input: &str) -> Parse {
        Parse { input }
    }

    pub fn literal(&self, word: &str) -> Option<Parse<'a>> {
        if self.input.starts_with(word) {
            Some(Parse {
                input: self.input.split_at(word.len()).1.trim_start(),
            })
        } else {
            None
        }
    }

    pub fn done<T, U>(&self, closure: T) -> Option<U>
    where
        T: FnOnce() -> U,
    {
        if self.input.is_empty() {
            Some(closure())
        } else {
            None
        }
    }

    pub fn done_or_err<T, U>(&self, done: T, command: &str) -> Result<U, String>
    where
        T: FnOnce() -> Result<U, String>,
    {
        self.done(done)
            .unwrap_or_else(|| Err(format!("Unexpected argument after \"{}\"", command)))
    }

    pub fn take_remaining<F, T>(&self, closure: F) -> T
    where
        F: FnOnce(&str) -> T,
    {
        closure(self.input)
    }
}
