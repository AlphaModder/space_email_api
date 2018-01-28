# space\_email\_api #
A [Rust](https://www.rust-lang.org) (via [`hyper`](https://hyper.rs/)) interface to [Space Email](https://space.galaxybuster.net), "an indirect communication platform that allows for a unique exchange of conversation over space and time." 

Full documentation coming soon.

## How to use ##
Access to space email is facilitated by the `space_email_api::SpaceEmailClient` struct. Its methods are relatively self-explanatory if you have used Space Email before.

## Coming soon ##
- More tests and debugging support.

## Changelog ##
- *0.1.2*: I'm back and I know rust waaaaay better! 
    - Add support for premium accounts, finally! Colors and other ranges should now work properly.
    - Rename `SpaceEmailClient::get_random_with_range` to `SpaceEmailClient::get_random_in_range`.
    - Add an (untested!) interface for starred emails. 
    - TODO: Add documentation!
    - TODO: Update dependencies.
- *0.1.1*:  Add Hash, Eq, etc. where appropriate and refactor accessibility of SpaceEmail fields to enforce their guarantees.
