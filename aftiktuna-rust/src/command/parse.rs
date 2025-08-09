use std::str::FromStr;

#[derive(Clone)]
pub struct Parse<'a> {
    input: &'a str,
    start: usize,
}

impl<'a> Parse<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, start: 0 }
    }

    pub fn empty<R, F: FnOnce() -> R>(&self, closure: F) -> Option<R> {
        if self.active_input().is_empty() {
            Some(closure())
        } else {
            None
        }
    }

    pub fn literal<R, F: FnOnce(Parse) -> R>(&self, word: &str, on_match: F) -> Option<R> {
        self.try_advance(word).map(on_match)
    }

    pub fn numeric<R, F: FnOnce(Parse, N) -> R, N: FromStr>(&self, on_match: F) -> Option<R> {
        let (word, parse) = self.next_word();
        str::parse(word).map(|number| on_match(parse, number)).ok()
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

    /// Matches the names of the provided objects against the start of the remainder of the command.
    /// If a match is found, the first closure is called with the matched object and a Parse for the new remainder.
    /// If none is found, the second closure is called with the remainder of the command.
    pub fn match_against<A, R, I, F, E>(self, iterable: I, success: F, failure: E) -> R
    where
        I: IntoIterator<Item = (String, A)>,
        F: FnOnce(Parse, A) -> R,
        E: FnOnce(&str) -> R,
    {
        iterable
            .into_iter()
            .find_map(|(name, object)| self.try_advance(&name).map(|parse| (parse, object)))
            .map_or_else(
                || failure(self.active_input()),
                |(parse, object)| success(parse, object),
            )
    }

    pub fn default_err<R>(&self) -> Result<R, String> {
        let consumed = self.consumed_input();
        let remaining = self.active_input();
        if consumed.is_empty() {
            Err(format!("Unexpected input: \"{remaining}\""))
        } else {
            Err(format!(
                "Unexpected input after \"{consumed}\": \"{remaining}\""
            ))
        }
    }

    fn active_input(&self) -> &'a str {
        &self.input[self.start..]
    }

    fn consumed_input(&self) -> &'a str {
        self.input[..self.start].trim_end()
    }

    fn try_advance(&self, word: &str) -> Option<Self> {
        let input = self.active_input();

        let input_word_advance = starts_with_ignore_ascii_case(input, word)?;

        let remainder = input.split_at(input_word_advance).1;
        if remainder.is_empty() {
            Some(self.advance_start(input_word_advance))
        } else if remainder.starts_with(' ') {
            Some(self.advance_start(input_word_advance + ' '.len_utf8()))
        } else {
            None
        }
    }

    fn next_word(&self) -> (&'a str, Self) {
        let input = self.active_input();
        for (i, char) in input.char_indices() {
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

macro_rules! first_match {
    ($($option:expr),+ $(,)?) => {
        $(
        if let Some(result) = $option {
            Some(result)
        }
        )else*
        else {
            None
        }
    }
}

fn starts_with_ignore_ascii_case(input: &str, prefix: &str) -> Option<usize> {
    let mut char_indices = input.char_indices();
    for char_to_match in prefix.chars() {
        let (_, char) = char_indices.next()?;
        if !char_to_match.eq_ignore_ascii_case(&char) {
            return None;
        }
    }
    Some(
        char_indices
            .next()
            .map(|(index, _)| index)
            .unwrap_or(input.len()),
    )
}

macro_rules! first_match_or {
    ($($option:expr),+ ; $err:expr) => {
        $(
        if let Some(result) = $option {
            result
        }
        )else*
        else {
            $err
        }
    }
}

pub(crate) use {first_match, first_match_or};
