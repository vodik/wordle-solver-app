# Wordle Solver

My toy application to have an excuse to experiment with WASM, Svelte,
and Parcel.

The solver should respect all inferrable constraints on letter
positions and frequency and offer suggestions by default that are  compatible with hard mode play.

## Challenges

- [ ] Fast filters
    - [X] Constant memory footprint without dynamic allocation
    - [ ] Filter concatenation (monoid)
    - [ ] Memoization (to support automated play)
- [ ] Frequency analysis
    - [ ] How often each letter appears in possible words
    - [ ] How often each letter appears in each position
- [ ] Offer suggestions based on frequency analysis
- [ ] Offer suggestions from fully automated play