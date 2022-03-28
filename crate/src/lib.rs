#![allow(dead_code)]

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

#[derive(Debug, Clone, Copy)]
enum Rule {
    Correct,
    Misplaced,
    Incorrect,
}

#[wasm_bindgen]
#[derive(Default, Debug)]
pub struct Filter {
    pos: usize,
    rules: [Option<(Rule, u8)>; 5],
    counts: [u8; 26],
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
        self.rules[self.pos] = Some((Rule::Correct, c));
        self.counts[position(c) as usize] += 1;
        self.includes |= mask(c);
        self.pos += 1;
    }

    #[wasm_bindgen(js_name = markMisplaced)]
    pub fn mark_misplaced(&mut self, c: char) {
        let c = c as u8;
        self.rules[self.pos] = Some((Rule::Misplaced, c));
        self.counts[position(c) as usize] += 1;
        self.includes |= mask(c);
        self.pos += 1;
    }

    #[wasm_bindgen(js_name = markIncorrect)]
    pub fn mark_incorrect(&mut self, c: char) {
        let c = c as u8;
        self.rules[self.pos] = Some((Rule::Incorrect, c));
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

    #[wasm_bindgen]
    pub fn filter(&self, filter: &Filter) -> Result<Dictionary, Error> {
        let includes = filter.includes;
        let excludes = filter.excludes & !includes;

        let mut correct = Vec::new();
        let mut incorrect = Vec::new();

        let mut counting = 0;
        let mut counts = Vec::new();

        for (pos, rule) in filter.rules.iter().enumerate() {
            match rule {
                Some((Rule::Correct, c)) => correct.push((*c, pos)),
                Some((Rule::Misplaced, c)) => incorrect.push((*c, pos)),
                Some((Rule::Incorrect, c)) => {
                    incorrect.push((*c, pos));

                    let mask = mask(*c);
                    if includes & mask != 0 {
                        let count = filter.counts[position(*c) as usize];
                        if counting & mask == 0 && count > 0 {
                            counts.push((*c, count as usize));
                            counting |= mask;
                        }
                    }
                }
                None => return Err(Error::new("Not enough rules provided")),
            }
        }

        let words = self
            .0
            .iter()
            .filter(|&&Word { letters, bitmap }| {
                bitmap & excludes == 0
                    && (includes == 0 || bitmap & includes == includes)
                    && correct.iter().all(|&(c, index)| letters[index] == c)
                    && incorrect.iter().all(|&(c, index)| letters[index] != c)
                    && counts
                        .iter()
                        .all(|&(c, count)| letters.iter().filter(|&&l| l == c).count() == count)
            })
            .cloned()
            .collect();

        Ok(Dictionary(words))
    }
}
