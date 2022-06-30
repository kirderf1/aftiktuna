pub struct Parse<'a> {
    input: &'a str,
    start: usize,
}

impl<'a> Parse<'a> {
    pub fn new(input: &str) -> Parse {
        Parse { input, start: 0 }
    }

    pub fn literal(&self, word: &str) -> Option<Parse<'a>> {
        let input = self.active_input();
        if input.starts_with(word) {
            let remainder = input.split_at(word.len()).1;
            if remainder.is_empty() {
                Some(Parse {
                    input: self.input,
                    start: self.start + word.len(),
                })
            } else if remainder.starts_with(" ") {
                Some(Parse {
                    input: self.input,
                    start: self.start + word.len() + " ".len(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn done<T, U>(&self, closure: T) -> Option<U>
    where
        T: FnOnce() -> U,
    {
        if self.active_input().is_empty() {
            Some(closure())
        } else {
            None
        }
    }

    pub fn done_or_err<T, U>(&self, done: T) -> Result<U, String>
    where
        T: FnOnce() -> Result<U, String>,
    {
        self.done(done).unwrap_or_else(|| {
            Err(format!(
                "Unexpected argument after \"{}\"",
                self.consumed_input()
            ))
        })
    }

    pub fn take_remaining<F, T>(&self, closure: F) -> T
    where
        F: FnOnce(&str) -> T,
    {
        closure(self.active_input())
    }

    fn active_input(&self) -> &'a str {
        self.input.split_at(self.start).1
    }

    fn consumed_input(&self) -> &'a str {
        self.input.split_at(self.start).0.trim_end()
    }
}
