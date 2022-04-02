<script>
  export let readonly = false;
  export let letters = "";
  export let status = Array(5).fill(null);

  const cycle = (index) => {
    if (readonly) {
      return;
    }

    if (status[index] === null) {
      status[index] = "yellow";
    } else if (status[index] === "yellow") {
      status[index] = "green";
    } else if (status[index] === "green") {
      status[index] = null;
    }
  };
</script>

<div class="letter-row">
  {#each { length: 5 } as _, i}
    <div
      class="letter-box {status[i]}"
      class:green={status[i] === "green"}
      class:yellow={status[i] === "yellow"}
      on:click={() => cycle(i)}
    >
      {letters[i] || ""}
    </div>
  {/each}
</div>

<style>
  .letter-row {
    display: flex;
  }

  .letter-box {
    user-select: none;
    border: 2px solid gray;
    border-radius: 3px;
    margin: 2px;
    font-size: 2.5rem;
    font-weight: 700;
    height: 3rem;
    width: 3rem;
    display: flex;
    justify-content: center;
    align-items: center;
    text-transform: uppercase;
  }

  .filled-box {
    border: 2px solid black;
  }

  .green {
    background-color: green;
  }

  .yellow {
    background-color: yellow;
  }
</style>
