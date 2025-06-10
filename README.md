
## Archived

The project was archived on 2025-06-11. It will no longer be maintained.


If I ever venture in the realm of games, game engines and/or computer graphics it will be as part of
a new project and with a new vision.

# GhostEngine
Yet another Rust game engine

GhostEngine is yet another data driven game engine written in Rust. The main purpose is for myself
to learn about game engine arhitecture and Rust and why not at some point to say, that's my thing.

This project in active development and it follows closely with the Youtube series with the same name:
https://www.youtube.com/playlist?list=PLQP1ylUm1ZZm13dCjcFdHGptr-Z-pMPMO

There may be development that's not logged there, but the main topics are present there in the format
of raw devlogs.

# License
MIT License

# Platforms
Until we get closer to `v1.0` we will focus on Linux support.
My main focus is on Fedora and the stable Rust compiler.

# Build
* Install Rust from https://rustup.rs/.
* Clone the code from https://github.com/DrOptix/GhostEngine
* Switch to the branch or tag you want, `develop` by default.
* There are no examples or apps for now, just run `./check.sh` to run all the tests, doc tests, clippy lints
  and code formating checks.

# Docs
For the moment generate them using:
`cargo doc --open`
