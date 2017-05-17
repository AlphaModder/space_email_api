# space\_email\_api #
A [Rust](https://www.rust-lang.org) (via [`hyper`](https://hyper.rs/)) interface to [Space Email](https://space.galaxybuster.net), "an indirect communication platform that allows for a unique exchange of conversation over space and time." 

Full documentation coming soon.

## How to use ##
Access to space email is facilitated by the `space_email_api::SpaceEmailClient` struct. Its methods are relatively self-explanatory if you have used Space Email before.

## Coming soon ##
- The ability to authenticate and use Space Email premium accounts through the API, and a proper enum for the range parameter of `SpaceEmailClient::get_random_with_range`. 
- More tests and debugging support.

