# `bbx_dsp`

This crate houses the foundational DSP code used for assembling complex chains of operations and functions.

## Design

To understand the design, it is important to learn these key ideas:

- The unit of data we are operating on is a `Sample`, which is typically a `f32` value
- A `Buffer` is a container for some number of `Sample`s
- A `Frame` is a container for some number of `Buffer`s, which each pertain to an audio channels (e.g. mono is 1 buffer, stereo is 2 buffers)
- Each `Node` processes or generates a `Frame` during every audio callback
- A (DSP) `Graph` is an ordering of `Node`s that is evaluated during every audio callback
