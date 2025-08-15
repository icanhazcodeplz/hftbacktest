## To rebuild rust code
```bash
cd py-hftbacktest
maturin develop 
```
This should create the file `py-hftbacktest/hftbacktest/_hftbacktest.cpython-313-darwin.so`

## TODO
 -  Try ROI setup to see if faster

## hftbacktest bugs
1. If exchange data feed has rows with identical timestamps, only processes last in the set

# Notes on Rust

IDE
 - Keyboard -> ergodox
 - check for Compiling with debug symbols
    - when compiling rust code it becomes machine code
    - need to tell to compile to keep line info in
 - new debugger gdb (C++ debugger for linux)
 - LLVM debugging tools
 - The Rust Book free online
 - Rust by Example
 - Ownership is "the big thing" coming from python
 - If worrying about lifetimes, then structural problem in code
 - Central tenat of rust
   - Only one piece of code can modify data at a time
   - If "&", then can only exist one at a time
 - Difficult to refactor in Rust
 - Python -> pencil
 - Rust -> stone tablet
 - py03 for python->rust
 - pydantic is very fast
 - pydantic can be used in rust (don't lose a ton of performance)
 - numba compiles into C and has a JIT
   - Will create faster version of code if used
 - Polars -> faster. built off of "datafusion". uses arrow.
   - wrapper around datafusion
   - SQL execution engine
   - Lazy by default
 - Altaire 
