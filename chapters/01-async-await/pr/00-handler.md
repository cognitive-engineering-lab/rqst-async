+++
file = "crates/miniserve/src/lib.rs"
end-line-side = "right"
end-line = 32
+++
Take a close look at the change to the `Handler` trait. Previously, a
handler was a function that took a request and returned a response. Now, a
handler is a function that takes a request and returns a *future* which
eventually returns a response.
