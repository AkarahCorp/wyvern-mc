# Wyvern
Wyvern is a Rust crate that allows you to make a Minecraft server.

# Get Started
To get started, add Wyvern to your `Cargo.toml`:
```toml
[dependencies]
wyvern_mc = { git = "https://github.com/akarahdev/wyvern-mc-rewrite.git" }
```

Once you have that, you will need to use the `ServerBuilder` type to create your server, obtaining an instance with `Server::builder`. Await `ServerBuilder::run` in order to start the server.

You also need to set up `Runtime`, otherwise the server will panic whenever it tries to do runtime-specific activities. If using `tokio`, you can use the `rt-tokio` crate flags and use the `Runtime::tokio` to automatically setup `Runtime` to use `tokio`.


# Examples
You can view examples in the `/examples` directory of this repository.

# Documentation
`todo!()`

# Prior Work
Wyvern was based off of many prior works:
- Valence (https://github.com/valence-rs/valence)
- PumpkinMC (https://github.com/Pumpkin-MC/Pumpkin)
- Minestom (https://github.com/Minestom/Minestom)
- Paper (https://github.com/PaperMC/Paper)

Wyvern has taken inspiration from many of these projects, but it does many things differently:
- Unlike Minestom and Paper, Wyvern is implemented in Rust
- Valence uses an ECS, Wyvern does not
- PumpkinMC attempts to reimplement vanilla behavior, Wyvern leaves all mechanics up to the developer, similar to Minestom