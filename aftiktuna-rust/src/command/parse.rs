pub struct Parse<'a> {
    input: &'a str,
    start: usize,
}

impl<'a> Parse<'a> {
    pub fn new(input: &str) -> Parse {
        Parse { input, start: 0 }
    }

    pub fn literal<R, F: FnOnce(Parse) -> R>(self, word: &str, closure: F) -> Partial<'a, R> {
        self.try_advance(word)
            .map(closure)
            .map(Partial::Matched)
            .unwrap_or(Partial::UnMatched(self))
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

    pub fn match_against<A, F, E, T>(self, vec: Vec<(String, A)>, success: F, failure: E) -> T
    where
        F: FnOnce(Parse, A) -> T,
        E: FnOnce(&str) -> T,
    {
        vec.into_iter()
            .fold(None, |previous, (name, object)| {
                previous.or_else(|| self.try_advance(&name).map(|parse| (parse, object)))
            })
            .map(|(parse, object)| success(parse, object))
            .unwrap_or_else(|| failure(self.active_input()))
    }

    fn active_input(&self) -> &'a str {
        self.input.split_at(self.start).1
    }

    fn consumed_input(&self) -> &'a str {
        self.input.split_at(self.start).0.trim_end()
    }

    fn try_advance(&self, word: &str) -> Option<Self> {
        let input = self.active_input();

        if !input.starts_with(word) {
            return None;
        }

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
