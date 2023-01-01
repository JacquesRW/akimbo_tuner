# akimbo_tuner

This repository contains a multithreaded implementation of Texel's Tuning Method in under 200 lines of code. It is used to tune
[akimbo](https://github.com/JacquesRW/akimbo)'s piece-square tables.

### Running
To compile, run ```cargo build --release```.
In order to run the tuner, requires a file named "set.epd" of positions, with the same format as the Zurichess dataset, in the same directory as the executable.
