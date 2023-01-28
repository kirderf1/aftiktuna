use std::str::FromStr;

#[derive(Clone)]
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
            .map_or(Partial::UnMatched(self), Partial::Matched)
    }

    pub fn done<R, F: FnOnce() -> R>(self, closure: F) -> Partial<'a, R> {
        if self.active_input().is_empty() {
            Partial::Matched(closure())
        } else {
            Partial::UnMatched(self)
        }
    }

    pub fn done_or_err<R, F: FnOnce() -> Result<R, String>>(self, done: F) -> Result<R, String> {
        if self.active_input().is_empty() {
            done()
        } else {
            Err(format!(
                "Unexpected argument after \"{}\"",
                self.consumed_input()
            ))
        }
    }

    pub fn take_remaining<R, F: FnOnce(&str) -> R>(self, closure: F) -> R {
        closure(self.active_input())
    }

    pub fn numeric<R, F: FnOnce(Parse, N) -> R, N: FromStr>(self, closure: F) -> Partial<'a, R> {
        let (word, parse) = self.next_word();
        str::parse(word)
            .map(|number| Partial::Matched(closure(parse, number)))
            .unwrap_or_else(|_| Partial::UnMatched(self))
    }

    pub fn match_against<A, F, E, R>(self, vec: Vec<(String, A)>, success: F, failure: E) -> R
    where
        F: FnOnce(Parse, A) -> R,
        E: FnOnce(&str) -> R,
    {
        vec.into_iter()
            .fold(None, |previous, (name, object)| {
                previous.or_else(|| self.try_advance(&name).map(|parse| (parse, object)))
            })
            .map(|(parse, object)| success(parse, object))
            .unwrap_or_else(|| failure(self.active_input()))
    }

    fn active_input(&self) -> &'a str {
        &self.input[self.start..]
    }

    fn consumed_input(&self) -> &'a str {
        self.input[..self.start].trim_end()
    }

    fn try_advance(&self, word: &str) -> Option<Self> {
        let input = self.active_input();

        if !input.starts_with(word) {
            return None;
        }

        let remainder = input.split_at(word.len()).1;
        if remainder.is_empty() {
            Some(self.advance_start(word.len()))
        } else if remainder.starts_with(' ') {
            Some(self.advance_start(word.len() + 1))
        } else {
            None
        }
    }

    fn next_word(&self) -> (&str, Parse) {
        let input = self.active_input();
        for (i, char) in input.chars().enumerate() {
            if char == ' ' {
                return (&input[..i], self.advance_start(i + 1));
            }
        }

        (input, self.clone())
    }

    fn advance_start(&self, length: usize) -> Self {
        Parse {
            input: self.input,
            start: self.start + length,
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

    pub fn match_against<A, F, E>(self, vec: Vec<(String, A)>, success: F, failure: E) -> R
    where
        F: FnOnce(Parse, A) -> R,
        E: FnOnce(&str) -> R,
    {
        match self {
            Partial::UnMatched(parse) => parse.match_against(vec, success, failure),
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
