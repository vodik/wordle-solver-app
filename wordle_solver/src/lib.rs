use js_sys::Error;
use std::{fmt, str};
use wasm_bindgen::prelude::*;

fn position(c: u8) -> u8 {
    c - b'a'
}

fn mask(c: u8) -> u32 {
    1 << position(c) as u32
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Word {
    letters: [u8; 5],
    bitmap: u32,
}

#[wasm_bindgen]
impl Word {
    #[wasm_bindgen(constructor)]
    pub fn new(word: &str) -> Result<Word, Error> {
        if word.len() != 5 {
            return Err(Error::new("Word must be 5 characters"));
        }

        let mut letters = [0u8; 5];
        let mut bitmap = 0;
        for (dst, c) in letters.iter_mut().zip(word.bytes()) {
            *dst = c;
            bitmap |= mask(c);
        }

        Ok(Word { letters, bitmap })
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: bytes are garanteed to come from a utf-8 valid string
        unsafe { write!(f, "{}", str::from_utf8_unchecked(&self.letters)) }
    }
}

#[derive(Clone, Copy)]
enum Limit {
    AtLeast,
    Exactly,
}

#[derive(Clone, Copy)]
struct Expect {
    expect: u8,
    limit: Limit,
}

impl Default for Expect {
    fn default() -> Self {
        Self {
            expect: 0,
            limit: Limit::AtLeast,
        }
    }
}

impl Expect {
    fn inc(&mut self) {
        self.expect += 1;
    }

    fn cap(&mut self) {
        self.limit = Limit::Exactly;
    }

    fn is_interesting(&self) -> bool {
        matches!(self.limit, Limit::Exactly) || self.expect > 1
    }

    fn check(&self, count: usize) -> bool {
        let count: u8 = count.try_into().unwrap();
        match self.limit {
            Limit::AtLeast => count >= self.expect,
            Limit::Exactly => count == self.expect,
        }
    }
}

#[derive(Default)]
struct Constraint {
    expect: Expect,
    must_have: u8,
    must_exclude: u8,
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Filter {
    constraints: [Option<Constraint>; 26],
    includes: u32,
    excludes: u32,
}

impl Filter {
    fn entry_mut(&mut self, c: u8) -> &mut Constraint {
        let cell = &mut self.constraints[position(c) as usize];
        if cell.is_none() {
            *cell = Some(Constraint::default());
        }
        cell.as_mut().unwrap()
    }

    fn constraints(&self) -> impl Iterator<Item = (&Constraint, u8)> + '_ {
        self.constraints
            .iter()
            .zip(b'a'..=b'z')
            .flat_map(|(cell, cur)| cell.as_ref().map(|state| (state, cur)))
    }
}

#[wasm_bindgen]
impl Filter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Filter {
        Self::default()
    }

    #[wasm_bindgen(js_name = markCorrect)]
    pub fn mark_correct(&mut self, c: char, pos: usize) {
        let c = c as u8;
        let pos_mask = 1 << pos;

        let state = self.entry_mut(c);
        state.expect.inc();
        state.must_have |= pos_mask;

        self.includes |= mask(c);
    }

    #[wasm_bindgen(js_name = markMisplaced)]
    pub fn mark_misplaced(&mut self, c: char, pos: usize) {
        let c = c as u8;
        let pos_mask = 1 << pos;

        let state = self.entry_mut(c);
        state.expect.inc();
        state.must_exclude |= pos_mask;

        self.includes |= mask(c);
    }

    #[wasm_bindgen(js_name = markIncorrect)]
    pub fn mark_incorrect(&mut self, c: char, pos: usize) {
        let c = c as u8;
        let pos_mask = 1 << pos;

        let state = self.entry_mut(c);
        state.expect.cap();
        state.must_exclude |= pos_mask;

        self.excludes |= mask(c);
    }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Dictionary(Vec<Word>);

#[wasm_bindgen]
impl Dictionary {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Dictionary {
        Self::default()
    }

    pub fn add(&mut self, word: &str) -> Result<(), Error> {
        let word = Word::new(word)?;
        self.0.push(word);
        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<String> {
        self.0.get(index).map(|word| word.to_string())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, word: &str) -> bool {
        if word.len() != 5 {
            return false;
        }

        let mut letters = [0u8; 5];
        for (dst, c) in letters.iter_mut().zip(word.bytes()) {
            *dst = c;
        }

        self.0.iter().any(|word| word.letters == letters)
    }

    pub fn filter(&self, filter: &Filter) -> Dictionary {
        let includes = filter.includes;
        let excludes = filter.excludes & !includes;

        let words = self
            .0
            .iter()
            .filter(|&Word { bitmap, .. }| {
                // We first apply a quick check to filter out words
                // that exclude all known mistakes and includes at
                // least one of each correct letter.
                //
                // This check is very fast and helps cut down the
                // amount of words we have to carefully consider.
                bitmap & excludes == 0 && (includes == 0 || bitmap & includes == includes)
            })
            .filter(|&Word { letters, .. }| {
                filter
                    .constraints()
                    .filter(|(constraint, _)| {
                        // Since the rough filter does a lot of the heavy lifting ahead of time, we
                        // can focus on only checking rules where we expect at least one match to
                        // be found.
                        constraint.expect.expect > 0
                    })
                    .all(|(constraint, cur)| {
                        // Now we can check all known positional constraints
                        let correct_positions = letters.iter().enumerate().all(|(pos, &letter)| {
                            let pos_mask = 1 << pos;
                            if letter != cur {
                                constraint.must_have & pos_mask == 0
                            } else {
                                constraint.must_exclude & pos_mask == 0
                            }
                        });

                        // And then check any necessay frequency contraints
                        //
                        // Frequency contains only need to be explicitly checked under a few
                        // conditions:
                        // 1. If we know the *exact* number of times a letter must apear
                        // 2. If it know the letter must apear more than once
                        //
                        // In all other cases, this constraint provides no additional information
                        // and can be skipped.
                        let correct_freq = !constraint.expect.is_interesting()
                            || constraint
                                .expect
                                .check(letters.iter().filter(|&&letter| letter == cur).count());

                        correct_positions && correct_freq
                    })
            })
            .cloned()
            .collect();

        Dictionary(words)
    }
}
