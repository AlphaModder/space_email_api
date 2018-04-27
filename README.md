# space\_email\_api #
A [Rust](https://www.rust-lang.org) (via [`hyper`](https://hyper.rs/)) interface to [space email](https://space.galaxybuster.net), "an indirect communication platform that allows for a unique exchange of conversation over space and time." 

Full documentation coming soon.

## How to use ##
Access to space email is facilitated by the `space_email_api::SpaceEmailClient` struct. Its methods are relatively self-explanatory if you have used Space Email before.

## Coming soon ##
- More tests and debugging support.

## Changelog ##
- *0.2.0*: I'm back and I know rust waaaaay better! Breaking changes abound.
    - Following `hyper`'s example, switch to a futures-based interface.
    - Have the user supply their own `hyper::Client` in `SpaceEmailClient::new`. This removes the dependency on `hyper-native-tls`.
    - Properly add standard traits to data types now that I understand things.
    - Add support for serde on data types, gated behind a feature.
    - Add support for premium accounts, finally! Colors and other ranges should now work properly (when logged in).
    - Rename `SpaceEmailClient::get_random_with_range` to `SpaceEmailClient::get_random_in_range`.
    - Rename `SpaceEmailColor` to `SpaceEmailStyle` and add `Admin` style.
    - Rename `SpaceEmailClient::get_id` to `SpaceEmailClient::get_by_id`.
    - Add an (untested!) interface for starred emails. 
    - Update dependencies.
    - TODO: Add documentation!
- *0.1.1*:  Add Hash, Eq, etc. where appropriate and refactor accessibility of SpaceEmail fields to enforce their guarantees.
