import { Dictionary } from '../wordle_solver/Cargo.toml'
const dictionary = new Dictionary();

import { solutions } from './words.json';
solutions.forEach((word) => dictionary.add(word));

import App from './components/App.svelte';
const app = new App({
  target: document.body,
  props: {
    wordList: dictionary,
  },
});

export default app;
