+++
title = "Integrate chatbot crate"
+++
## Task

### Functionality

In {{02-join.pr}}, a colleague has implemented a more intelligent chatbot in the `chatbot` crate. Your task is to integrate the chatbot into the `/chat` route handler of the application server. Specifically, you should use `chatbot::query_chat` on the current conversation to generate a list of possible responses, and then use `chatbot::gen_random_number` to randomly select from amongst the list of responses. You will need to add the `chatbot` dependency to `crates/server/Cargo.toml`.

### Performance

These two functions are I/O intensive, meaning they each spend 2 seconds performing non-blocking operations before returning[^sleep]. You should minimize the latency of the chatbot, i.e., return a response from `/chat` within 2 seconds rather than 4. Try querying the chatbot in the frontend to observe the latency of the backend server.

## Background

### Futures are values

A future is a value you can assign to a variable and move around. For example, you could create a future and later await it like this:

```rust
async fn print_hello() {
  println!("Hello");
}

async fn greeting() {
  let fut = print_hello();
  println!("World!");
  fut.await;
}
```

When you call `greeting()`, it will actually print `World` and *then* `Hello`. *Creating* the future by calling `print_hello()` does not *start* the future, which is different than how futures work in many other languages. The future is not started until the first time it is awaited.

### The `join!` macro

If you want two futures to run concurrently on the same thread, one method is to use Tokio's [`join!`] macro. This macro will attempt to start all the futures, and then await them all to completion. For example:

```rust
async fn zero() { 0 }
async fn one() { 1 }
async fn both() {
  let a = zero();
  let b = one();
  let values = join!(a, b);
  assert_eq!(values, (0, 1));
}
```

[^sleep]:  A function being I/O-intensive is simulated here by using [`tokio::time::sleep`], but in practice could be things like reading a file or querying a database. The key thing is that you're waiting for something to happen, and the CPU does not need to perform computations to make that thing happen.

[`tokio::time::sleep`]: https://docs.rs/tokio/1.39.2/tokio/time/fn.sleep.html
[`join!`]: https://docs.rs/tokio/1.39.2/tokio/macro.join.html


