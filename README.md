# Rust JIT demo

This is a demo project showing my solution for converting any rust to LLVM (including linking compiled modules together), and then passing it to the LLVM JIT compiler.

This approach to JIT appears to be the best out there at the moment, and it's a similar approach to what projects like PostgreSQL do, except with full no_std Rust support.

Below are reproduction instructions, but talking slides are available in `_slides/SLIDES.md`.

**NOTE:** This project is NOT production ready or safe in any way, and probably has many LLVM related memory leaks where I don't free the LLVM objects properly. It should only be used as reference for the general approach.

### Demo problem

The problem this demo solves is having a bunch of "user" structs with many string fields, and a custom complex filter with many conditions that checks for matching users.

This problem isn't the best for JIT (because of the relatively low number of conditions and dispatch required), but it does demonstrate a performance improvement over the interpreted approach when using the JIT.

The mock users list is found in `./data.json`, and the filters the code runs are in `./runner/src/lib.rs`.

## Setup
**Note:** this is hardcoded for x86_64 linux. If you want to attempt ARM, in theory you could search for each mention of "x86" in the codebase and fix accordingly.

Ensure that you have the correct rust toolchains (that this project was last tested with):
```bash
$ TOOLCHAIN=$(cat rust-toolchain)
$ rustup toolchain install ${TOOLCHAIN}-x86_64-unknown-linux-gnu
$ rustup component add rust-src --toolchain ${TOOLCHAIN}-x86_64-unknown-linux-gnu
```

### Ensuring you have LLVM

You need LLVM 17 installed, and/or added to your `LLVM_SYS_170_PREFIX` variable. If you don't have it installed, you can build it yourself (see below).

Initializing/Compiling LLVM
```bash
# Download latest LLVM 17 source
$ wget https://github.com/llvm/llvm-project/releases/download/llvmorg-17.0.6/llvm-project-17.0.6.src.tar.xz

# Extract
$ tar -xf llvm-project-17.0.6.src.tar.xz
$ cd llvm-project-17.0.6.src

# Create and initialize build directory. This requires cmake and ninja.
$ mkdir build_release
$ cmake -S llvm -B build_release -G Ninja -DLLVM_ENABLE_PROJECTS='llvm' -DLLVM_TARGETS_TO_BUILD='X86' -DCMAKE_BUILD_TYPE=Release

# Build LLVM. This might take a while.
$ cmake --build build_release
```

Then add it to your `LLVM_SYS_170_PREFIX` variable for your IDE:
```bash
$ export LLVM_SYS_170_PREFIX=path/to/src/build_release
```
**IMPORTANT:** If you're using VSCODE, you must change the `LLVM_SYS_170_PREFIX` variable in the `.vscode/settings.json` file as well.

## Project structure

This project has the following packages:
- `./compile`: This project is more of a script, but it runs a compile command on `./functions` and performs all of the linking logic.
- `./functions`: This is where all of the functions are defined. More details about the structure of this are in the slides, but any no_std (with alloc) rust code can go in here, and in the dependencies of this project. E.g. you could import something like `nalgebra` or `heapless` or any other crate that supports no_std, and it will compile just fine (in theory).
- `./shared`: a no_std crate that is used by `./functions` and `./runner` to share code between them. It has an optional `std` feature to enable things like serde and debug on the structs it exports.
- `./runner`: This is the runner project. It takes the LLVM IR files generated by `./compile` and joins the exported functions together in whatever way needed, then optimizes and JIT compiles them. It then runs the functions and prints the results.

## Running the demo

First, you need to re-generate the LLVM IR files for the functions crate. To do that, run:
```bash
$ cargo run --package=compile
```

This should create 2 files: `./functions/compiled.bc` and `./functions/compiled.ll`. Only the `.bc` file is used as it's more efficient for LLVM to parse, but the `.ll` file is identical but in human readable form.

Then, you can run the runner project:
```bash
$ cargo run --package=runner
```

If `Interpreted len` and `JIT len` match, then the JIT correctly reflected the interpreted code for this test.

2 files should be created in the root of the project: `jit.ll` and `jit_opt.ll`. These are the resulting IR files from the JIT process, with the first one being the unoptimized version (raw after building the custom function), and the second one being the optimized version.

To benchmark, there's also `cargo bench` if you have criterion installed.
