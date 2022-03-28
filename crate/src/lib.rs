#![allow(dead_code)]

use js_sys::Error;
use std::{fmt, str};
use wasm_bindgen::prelude::*;

fn mask(c: u8) -> u32 {
    let shift = c - b'a';
    1 << shift as u32
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

type Rule = (u8, usize);

#[wasm_bindgen]
#[derive(Default)]
pub struct Filter {
    correct: Vec<Rule>,
    incorrect: Vec<Rule>,
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
    pub fn mark_correct(&mut self, c: char, pos: usize) {
        let c = c as u8;
        self.correct.push((c, pos));
        self.includes |= mask(c);
    }

    #[wasm_bindgen(js_name = markMisplaced)]
    pub fn mark_misplaced(&mut self, c: char, pos: usize) {
        let c = c as u8;
        self.incorrect.push((c, pos));
        self.includes |= mask(c);
    }

    #[wasm_bindgen(js_name = markIncorrect)]
    pub fn mark_incorrect(&mut self, c: char, pos: usize) {
        let c = c as u8;
        self.incorrect.push((c, pos));
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

    #[wasm_bindgen]
    pub fn add(&mut self, word: &str) -> Result<(), Error> {
        let word = Word::new(word)?;
        self.0.push(word);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn peek(&self) -> Option<String> {
        self.0.first().map(|word| word.to_string())

    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[wasm_bindgen]
    pub fn filter(&self, filter: &Filter) -> Dictionary {
        let includes = filter.includes;
        let excludes = filter.excludes & !includes;

        let words = self
            .0
            .iter()
            .filter(|word| {
                word.bitmap & excludes == 0
                    && (includes == 0 || word.bitmap & includes == includes)
                    && filter
                        .correct
                        .iter()
                        .all(|&(c, index)| word.letters[index] == c)
                    && filter
                        .incorrect
                        .iter()
                        .all(|&(c, index)| word.letters[index] != c)
            })
            .cloned()
            .collect();

        Dictionary(words)
    }
}
