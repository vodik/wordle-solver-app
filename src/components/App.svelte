<script>
  import { Filter } from "../../crate/Cargo.toml";
  import Row from "./Row.svelte";

  export let wordList;
  let words = wordList;
  $: possibilities = words.len();
  $: empty = words.isEmpty();
  let position = 0;

  let input = words.get(position);
  let status = Array(5).fill(null);
  let history = [];

  const filterList = (guess) => {
    const filter = new Filter();

    status.forEach((mark, index) => {
      const letter = guess[index];
      if (mark === "green") {
        filter.markCorrect(letter);
      } else if (mark === "yellow") {
        filter.markMisplaced(letter);
      } else {
        filter.markIncorrect(letter);
      }
    });

    history = [...history, { words, position, input, status }];

    position = 0;
    words = words.filter(filter);
    input = words.get(position);
    status = status.map((mark) => (mark !== "green" ? null : "green"));
  };

  const handleKeydown = (event) => {
    if (empty) {
      return;
    }

    const { key } = event;

    if (key === "Backspace" && input.length > 0) {
      input = input.slice(0, -1);
    } else if (key === "Enter" && input.length == 5) {
      filterList(input);
      event.preventDefault();
    } else if (key === "ArrowLeft") {
      prevWord();
    } else if (key === "ArrowRight") {
      nextWord();
    } else if (input.length < 5) {
      let found = key.match(/[a-z]/gi);
      if (found && found.length === 1) {
        input += found;
      }
    }
  };

  $: atFirstWord = position === 0;
  const prevWord = () => {
    position = Math.max(position - 1, 0);
    input = words.get(position);
  };

  $: atLastWord = position === possibilities - 1;
  const nextWord = () => {
    position = Math.min(position + 1, possibilities - 1);
    input = words.get(position);
  };

  const reset = () => {
    words = wordList;
    position = 0;
    input = words.get(position);
    status = Array(5).fill(null);

    history = [];
  }

  const undo = () => {
    const state = history.pop();
    history = history;

    words = state.words;
    position = status.position;
    input = state.input;
    status = state.status;
  };
</script>

<svelte:window on:keydown={handleKeydown} />

<button on:click={reset}>Reset</button>
<button disabled={history.length === 0} on:click={undo}>Undo</button>

<div id="game-board">
  {#each history as {input, status}}
    <Row letters={input} {status} readonly />
  {/each}
  {#if empty}
    No results found
  {:else}
    <button disabled={atFirstWord} on:click={prevWord}>&lt;</button>
    <Row letters={input} {status} />
    <button disabled={atLastWord} on:click={nextWord}>&gt;</button>
    <p>{possibilities} possibilities</p>
  {/if}
</div>

<style>
  #game-board {
    display: flex;
    align-items: center;
    flex-direction: column;
  }
</style>
