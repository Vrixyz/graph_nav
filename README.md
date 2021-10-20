# Pitch

Games sometimes use graphs to navigate through a strategic layer, secondary to what is decribed as the **main** gameplay. What if that graph navigation was the main gameplay ?

# Vision

- Rogueli*kt*e
- Move in an infinite graph map made of "rooms"
- Discover room types
- Make Strategic choices
- Die and Improve

# Tech

- Hierarchy of the project from https://matklad.github.io/2021/08/22/large-rust-workspaces.html
- Web build relies on rust version `nightly-2021-10-05` because of Rocket https://github.com/rust-lang/rust/issues/89935#issuecomment-945037448
- Web build uses [cargo-make](https://sagiegurari.github.io/cargo-make/) then github pages
