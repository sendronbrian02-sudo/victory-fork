# Speed pass — what changed in this ZIP, and what it does not do

This is the state of the fork after one pass over the frame path. Everything
here compiles (`cargo build --release -p victory`, `target/release/victory.exe`)
and the workspace's tests pass (`cargo test --workspace --release`:
347 tests, 0 failures).

## What changed

The layer already carried a set of speed switches, each written and measured
on the test machine (Windows 11, i7-14700KF, RTX 4070 12 GB), and all of them
were **off**: a plain run got the slow path, and the fast one needed a list of
environment variables that no player would type. This pass makes the ones that
were measured as paying the default, so a plain run gets them.

| Switch | Now | What it removes from a frame |
|--------|-----|------------------------------|
| `VICTORY_LATE_WAIT=3` | default | the turn-taking between the submitting thread and the card: it used to be prepare 134 ms, then wait for the card 194 ms, then copy back 78 ms, one after the other. The title is now told a submission's fences and bottom-of-pipe values when the card has run it |
| `VICTORY_RELAY=1` | default | the copy-back of what shaders wrote ran on the submitting thread, inside the submission lock; it now runs beside it |
| `VICTORY_ASYNC_SUBMIT=1` | default | the title's render thread spent 257 of 292 ms of a frame inside the submit path; submissions are prepared and sent by a thread of the layer now |
| `VICTORY_QUEUE2=1` | default | fetches of what shaders wrote go to a copy queue: the copy-back thread's wait fell from about 125 to 7 ms a frame |
| `VICTORY_INDIRECT_WINDOW=1` (+ its per-draw check) | default | indirect draw arguments were clamped record by record; now one window at a time, with each draw's record re-clamped and compared on the card so a stale argument cannot slip through |
| `VICTORY_COPY_PLAIN=1` | default | copies of title memory went through the kernel: the submit thread's kernel time fell from 44 to 20 ms a frame with copies made in place |
| `VICTORY_VIEW_KNOWN=1` | default | 338-450 failed write-watch askings and 6.5-7.8 ms of `VirtualQuery` a frame, on view memory |
| `VICTORY_WIDE_THREADS` | 8 (was a cap of 6) | the kept threads that do a submission's refreshes, the snapshots and compares of declared ranges and the merge of what shaders wrote. Six of them were the ceiling of a 136 ms frame whose "copies" and "copy-back" were 81 ms |
| `VICTORY_FULL_CARD_PASS` | on | while the card has no room for new textures, the texture pass walked its 18,000 slots every frame to learn the same thing; it is left out now |

Nothing else in the renderer was rewritten: the translation pipeline, the
shader work, the ray-tracing work in progress and the picture are as they were.

## What this does not do: 40 fps

**This does not reach 40 frames a second, and nothing in this pass pretends
to.** The honest numbers from the fork's own measurements:

- before the pass, the world ran at 2-4 fps (420-450 ms a frame), 240-350 ms
  with the fast switches on;
- with the switches on, the same first room was measured at 6-7 fps, and
  7-8 fps in the first rooms, 10-12 in their lighter views;
- the frame is linear in draws at about 32 µs a draw all-in, and the title
  draws 4,300 to 10,000 a frame in the world: 40 fps would need 25 ms a frame
  where the card alone spends 102 ms in the title's own lists.

So this pass is worth roughly what the switches were worth together (a factor
of two to three on the frames the fork measured), not the factor of ten to
twenty that 40 fps asks for. Reaching it needs work the fork has not done:

1. **The per-record clamp in the title's own lists is most of the card's
   time** (102 of 130 ms). Windows take a large part of it, but the remaining
   clamp per draw is still on the card. This needs a different way of
   emulating the console's per-record argument fetch, not a switch.
2. **The copies between the PC's memory and the card's are most of the
   submit thread's time** even with the relay and the copy queue: the title
   rewrites large parts of its memory every frame, and every byte of it that
   shaders read is copied. `VICTORY_INPLACE` (title memory opened as a
   graphics heap, so buffers are used where they are) is the way out; only its
   first stage exists.
3. **Shadowed areas are black**: the game's ray-traced bounce light and its
   ray-traced lamp and sun shadows do not run yet, so anything the sun does
   not hit directly is black. That is picture, not speed, but it is why the
   fork is not playable yet.

## If a run looks wrong

The three switches that change what the title observes are the late wait, the
relay and the asynchronous submit. Turn them off in this order:

```
set VICTORY_LATE_WAIT=0     :: the old turn-taking with the card
set VICTORY_RELAY=0         :: copy-back on the submitting thread
set VICTORY_ASYNC_SUBMIT=0  :: submissions prepared by the submitting thread
set VICTORY_INDIRECT_WINDOW=0   :: no draw windows at all
```

Each is described in a comment where it is read, in `crates/victory-gpu/src`.
