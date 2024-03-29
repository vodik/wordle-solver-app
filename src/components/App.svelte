<script>
  import { Filter } from "../../wordle_solver/Cargo.toml";
  import Row from "./Row.svelte";

  export let wordList;
  let list = wordList;
  $: possibilities = list.len();
  $: empty = list.isEmpty();
  let position = 0;

  let input = list.get(position);
  let status = Array(5).fill(null);
  let history = [];

  $: inputFull = input && input.length === 5;
  $: validWord = inputFull && wordList.contains(input);
  $: possibleTarget = inputFull && list.contains(input);

  const filterList = (guess) => {
    const filter = new Filter();

    status.forEach((mark, index) => {
      const letter = guess[index];
      if (mark === "green") {
        filter.markCorrect(letter, index);
      } else if (mark === "yellow") {
        filter.markMisplaced(letter, index);
      } else {
        filter.markIncorrect(letter, index);
      }
    });

    history = [...history, { input, status, position, list }];

    position = 0;
    list = list.filter(filter);
    input = list.get(position);
    status = status.map((mark) => (mark !== "green" ? null : "green"));
  };

  const handleKeydown = (event) => {
    if (empty) {
      return;
    }

    const { key } = event;
    if (key === "Backspace" && input.length > 0) {
      input = input.slice(0, -1);
    } else if (key === "Enter" && inputFull) {
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
  $: atLastWord = position === possibilities - 1;

  const prevWord = () => {
    position = Math.max(position - 1, 0);
    input = list.get(position);
  };

  const nextWord = () => {
    position = Math.min(position + 1, possibilities - 1);
    input = list.get(position);
  };

  const reset = () => {
    list = wordList;
    position = 0;
    input = list.get(position);
    status = Array(5).fill(null);

    history = [];
  };

  const undo = () => {
    const state = history.pop();
    history = history;

    list = state.list;
    position = state.position;
    input = state.input;
    status = state.status;
  };
</script>

<svelte:window on:keydown={handleKeydown} />

<button on:click={reset}>Reset</button>
<button disabled={history.length === 0} on:click={undo}>Undo</button>

<div id="game-board">
  {#each history as { input, status }}
    <Row letters={input} {status} readonly />
  {/each}
  {#if empty}
    No results found
  {:else}
    <div id="input-row">
    <button disabled={atFirstWord} on:click={prevWord}>&lt;</button>
    <Row letters={input} {status} />
    <button disabled={atLastWord} on:click={nextWord}>&gt;</button>
    </div>
    <p>{possibilities.toLocaleString()} possibilities</p>
    {#if inputFull && !validWord}
      <p>Not a valid word</p>
    {:else if inputFull && !possibleTarget}
      <p>Word is valid but not a possible solution</p>
    {/if}
  {/if}
</div>

<style>
  #game-board {
    display: flex;
    align-items: center;
    flex-direction: column;
  }

  #input-row {
    display: flex;
    flex-direction: row;
  }

  #input-row button {
    margin: 5px;
  }
</style>
