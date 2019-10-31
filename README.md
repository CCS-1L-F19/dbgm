# dbgm
`dbgm` is a tool for managing a library of Windows desktop backgrounds. When it is finished, it will support the ability to import backgrounds from a variety of sources, and to edit these backgrounds to best fit a screen while preserving the originals. 

## Installation
In its unfinished state, the only way to install `dbgm` is to build it locally. The steps to do so are as follows:

1. `dbgm` is written in Rust, and requires a Rust toolchain to build. If Rust is not already present on your system, you can install it by following the instructions [here](https://www.rust-lang.org/tools/install).

2. Once Rust is installed, clone this repository:

    ```
    git clone https://github.com/CCS-F19-1L/dbgm.git
    cd dbgm
    ```

3. Then, to build, execute the command `cargo build`. To run the application, execute `cargo run`. 

## Development Status
Currently, `dbgm` is still in development, and basic features are not present. Progress is tracked on the [kanban board](https://github.com/CCS-1L-F19/dbgm/projects/1), and some design goals can be found in [DESIGN.md](https://github.com/CCS-1L-F19/dbgm/blob/master/DESIGN.md).

## Documentation
Documentation is planned, but not currently a priority due to the unfinished and unstable state of the project. 

## License
`dbgm` is distributed under the terms of the MIT License.
