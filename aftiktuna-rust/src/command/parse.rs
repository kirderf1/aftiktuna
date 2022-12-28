pub struct Parse<'a> {
    input: &'a str,
    start: usize,
}

impl<'a> Parse<'a> {
    pub fn new(input: &str) -> Parse {
        Parse { input, start: 0 }
    }

    pub fn literal<R, F: FnOnce(Parse) -> R>(self, word: &str, closure: F) -> Partial<'a, R> {
        let input = self.active_input();

        if !input.starts_with(word) {
            return Partial::UnMatched(self);
        }

        let remainder = input.split_at(word.len()).1;
        if remainder.is_empty() {
            Partial::Matched(closure(Parse {
                input: self.input,
                start: self.start + word.len(),
            }))
        } else if remainder.starts_with(" ") {
            Partial::Matched(closure(Parse {
                input: self.input,
                start: self.start + word.len() + " ".len(),
            }))
        } else {
            Partial::UnMatched(self)
        }
    }

    pub fn done<T, U>(self, closure: T) -> Partial<'a, U>
    where
        T: FnOnce() -> U,
    {
        if self.active_input().is_empty() {
            Partial::Matched(closure())
        } else {
            Partial::UnMatched(self)
        }
    }

    pub fn done_or_err<T, U>(self, done: T) -> Result<U, String>
    where
        T: FnOnce() -> Result<U, String>,
    {
        if self.active_input().is_empty() {
            done()
        } else {
            Err(format!(
                "Unexpected argument after \"{}\"",
                self.consumed_input()
            ))
        }
    }

    pub fn take_remaining<F, T>(self, closure: F) -> T
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

pub enum Partial<'a, R> {
    UnMatched(Parse<'a>),
    Matched(R),
}

impl<'a, R> Partial<'a, R> {
    pub fn literal<F: FnOnce(Parse) -> R>(self, word: &str, closure: F) -> Self {
        match self {
            Partial::UnMatched(parse) => parse.literal(word, closure),
            Partial::Matched(r) => Partial::Matched(r),
        }
    }

    pub fn or_else_remaining<F: FnOnce(&str) -> R>(self, closure: F) -> R {
        match self {
            Partial::UnMatched(parse) => parse.take_remaining(closure),
            Partial::Matched(r) => r,
        }
    }
}

impl<'a, T, E> Partial<'a, Result<T, E>> {
    pub fn or_else_err<F: FnOnce() -> E>(self, closure: F) -> Result<T, E> {
        match self {
            Partial::UnMatched(_) => Err(closure()),
            Partial::Matched(r) => r,
        }
    }
}
