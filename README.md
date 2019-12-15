# space\_email\_api #
A [Rust](https://www.rust-lang.org) (via [`hyper`](https://hyper.rs/)) interface to [space email](https://space.galaxybuster.net), "an indirect communication platform that allows for a unique exchange of conversation over space and time." 

Documentation is a work in progress.

## How to use ##
Access to space email is facilitated by the `space_email_api::SpaceEmailClient` struct. Its methods are relatively self-explanatory if you have used Space Email before. You may refer to [its documentation](https://docs.rs/space_email_api/0.3.0/space_email_api/struct.SpaceEmailClient.html) for details.

## Coming soon ##
- More tests and debugging support.
- Documentation on types other than `SpaceEmailClient`, though they should be relatively self-explanatory.

## Changelog ##
- *0.3.0*: Async-await support!
    - Drop `hyper` for `reqwest`, which handles a lot of stuff we previously had to think about automatically.
    - Rewrite `SpaceEmailClient` entirely to use `std::future::Future` and async-await.
    - Add `EmailId` type to represent the id of a `SpaceEmail`. It implements `From<u32>` and `Into<u32>`.
        - `SpaceEmailClient::get_by_id` now takes `impl Into<EmailId>`.
        - `SpaceEmail::id` now returns `EmailId`.
        - `SpaceEmailClient::star` and `SpaceEmailClient::unstar` now take `impl Into<EmailId>` instead of `&SpaceEmail`.
        - `SpaceEmailClient::starred_emails` now returns an iterator of `EmailId` rather than grabbing the emails automatically. This gives more flexibility to the user in handling errors, should they occur.
    - Remove `SpaceEmailError::Encoding` variant, since it can no longer occur.
    - Remove dependencies on `lazy_static` and `url`.
    - Document the methods of `SpaceEmailClient`.
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
- *0.1.1*:  Add Hash, Eq, etc. where appropriate and refactor accessibility of SpaceEmail fields to enforce their guarantees.
