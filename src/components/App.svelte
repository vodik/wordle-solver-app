<script>
  import { Filter } from "../../crate/Cargo.toml";
  import Row from "./Row.svelte";

  export let wordList;
  let words = wordList;
  $: possibilities = words.len();

  let position = 0;
  $: input = words.get(position);
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

    history = [...history, { input, status }];

    words = words.filter(filter);
    status = status.map((mark) => (mark !== "green" ? null : "green"));
  };

  const handleKeydown = ({ key }) => {
    if (key === "Backspace" && input.length > 0) {
      input = input.slice(0, -1);
    } else if (key === "Enter") {
      filterList(input);
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
  };

  $: atLastWord = position === possibilities - 1;
  const nextWord = () => {
    position = Math.min(position + 1, possibilities - 1);
  };
</script>

<svelte:window on:keydown={handleKeydown} />

<div id="game-board">
  {#each history as row}
    <Row letters={row.input} status={row.status} />
  {/each}
  <button disabled={atFirstWord} on:click={prevWord}>&lt;</button>
  <Row letters={input} {status} />
  <button disabled={atLastWord} on:click={nextWord}>&gt;</button>
  <p>{possibilities} possibilities</p>
</div>

<style>
  #game-board {
    display: flex;
    align-items: center;
    flex-direction: column;
  }
</style>
