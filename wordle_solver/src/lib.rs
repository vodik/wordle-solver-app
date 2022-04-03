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
struct Frequency {
    expect: u8,
    limit: Limit,
}

impl Default for Frequency {
    fn default() -> Self {
        Self {
            expect: 0,
            limit: Limit::AtLeast,
        }
    }
}

impl Frequency {
    fn inc(&mut self) {
        self.expect += 1;
    }

    fn cap(&mut self) {
        self.limit = Limit::Exactly;
    }

    fn is_zero(&self) -> bool {
        self.expect == 0
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
struct State {
    freq: Frequency,
    must_have: u8,
    must_exclude: u8,
}

impl State {
    fn is_empty(&self) -> bool {
        self.freq.is_zero() && self.must_have == 0 && self.must_exclude == 0
    }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Filter {
    pos: u8,
    state: [State; 26],
    includes: u32,
    excludes: u32,
}

#[wasm_bindgen]
impl Filter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Filter {
        Self::default()
    }

    #[wasm_bindgen(js_name = markCorrect)]
    pub fn mark_correct(&mut self, c: char) {
        let c = c as u8;

        let state = &mut self.state[position(c) as usize];
        state.freq.inc();
        state.must_have |= 1 << self.pos;

        self.includes |= mask(c);
        self.pos += 1;
    }

    #[wasm_bindgen(js_name = markMisplaced)]
    pub fn mark_misplaced(&mut self, c: char) {
        let c = c as u8;

        let state = &mut self.state[position(c) as usize];
        state.freq.inc();
        state.must_exclude |= 1 << self.pos;

        self.includes |= mask(c);
        self.pos += 1;
    }

    #[wasm_bindgen(js_name = markIncorrect)]
    pub fn mark_incorrect(&mut self, c: char) {
        let c = c as u8;

        let state = &mut self.state[position(c) as usize];
        state.freq.cap();
        state.must_exclude |= 1 << self.pos;

        self.excludes |= mask(c);
        self.pos += 1;
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

    #[wasm_bindgen]
    pub fn add(&mut self, word: &str) -> Result<(), Error> {
        let word = Word::new(word)?;
        self.0.push(word);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn get(&self, index: usize) -> Option<String> {
        self.0.get(index).map(|word| word.to_string())
    }

    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[wasm_bindgen]
    pub fn filter(&self, filter: &Filter) -> Result<Dictionary, Error> {
        if filter.pos != 5 {
            return Err(Error::new("Not enough rules provided"));
        }

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
                    .state
                    .iter()
                    .zip(b'a'..=b'z')
                    .filter(|(state, _)| !state.is_empty())
                    .all(|(state, cur)| {
                        // Now we can check all known positional constraints
                        let correct_positions = letters.iter().enumerate().all(|(pos, &letter)| {
                            let mask = 1 << pos;
                            if letter != cur {
                                state.must_have & mask == 0
                            } else {
                                state.must_exclude & mask == 0
                            }
                        });

                        // And then check the frequency contraints.
                        //
                        // We can skip any zero constraints as they'll
                        // have already been solved by the rought
                        // bitmap filter above.
                        let correct_count = state.freq.is_zero()
                            || state
                                .freq
                                .check(letters.iter().filter(|&&letter| letter == cur).count());

                        correct_positions && correct_count
                    })
            })
            .cloned()
            .collect();

        Ok(Dictionary(words))
    }
}
