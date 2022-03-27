import App from './components/App.svelte';

import { Dictionary } from '../crate/Cargo.toml'

const dictionary = new Dictionary();
import words from './words.json';
words.forEach((word) => dictionary.add(word));

const app = new App({
  target: document.body,
  props: {
    wordList: dictionary,
  },
});

export default app;
