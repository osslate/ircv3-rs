#![doc(html_logo_url="https://raw.githubusercontent.com/sigkell/ircv3-rs/master/logo.svg")]
/// A set of parsing functions for the IRC protocol, and related formats.
/// 
/// # Design
/// 
/// The parsers within this module are designed not to copy memory, unless
/// necessary (such as escaping encoded values). When designing the
/// implementations, care was made to optimise for low memory usage, as well
/// as correctness, based on recent/evolving standards where possible.
pub mod parsers;
/// Utilities to help serialize IRC messages before sending to a server.
pub mod serialize;
/// A full implementation of the core logic needed for an IRC client.
#[allow(dead_code)]
pub mod client;