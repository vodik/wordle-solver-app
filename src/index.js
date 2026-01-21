import init, { Dictionary } from '../wordle_solver/pkg/wordle_solver.js'
import { solutions } from './words.json';
import App from './components/App.svelte';

await init();

const dictionary = new Dictionary();
solutions.forEach((word) => dictionary.add(word));

const app = new App({
  target: document.body,
  props: {
    wordList: dictionary,
  },
});

export default app;
