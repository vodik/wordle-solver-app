<script>
  import { Filter } from "../../crate/Cargo.toml";
  import Row from "./Row.svelte";

  export let wordList;
  let words = wordList;

  $: input = words.peek();
  let history = [];
  $: possibilities = words.len();
  let status = Array(5).fill(null);

  const filterList = (guess) => {
    const filter = new Filter();

    status.forEach((mark, index) => {
      const letter = guess[index];
      if (mark === null) {
        filter.markIncorrect(letter, index);
      } else if (mark === "green") {
        filter.markCorrect(letter, index);
      } else if (mark === "yellow") {
        filter.markMisplaced(letter, index);
      }
    });

    history = [...history, {input, status}];
    words = words.filter(filter);
    status = status.map((mark) => (mark !== "green" ? null : "green"));
  };

  const handleKeydown = ({ key }) => {
    if (key === "Backspace" && input.length > 0) {
      input = input.slice(0, -1);
    } else if (key === "Enter") {
      filterList(input);
    } else if (input.length < 5) {
      let found = key.match(/[a-z]/gi);
      if (found && found.length === 1) {
        input += found;
      }
    }
  };
</script>

<svelte:window on:keydown={handleKeydown} />

<div id="game-board">
  {#each history as row }
    <Row letters={row.input} status={row.status} />
  {/each}
  <Row letters={input} {status} />
  <p>{possibilities} possibilities</p>
</div>

<style>
  #game-board {
    display: flex;
    align-items: center;
    flex-direction: column;
  }
</style>
