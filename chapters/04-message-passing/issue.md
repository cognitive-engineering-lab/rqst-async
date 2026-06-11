+++
title = "Update to new chatbot API"
+++
## Task

### Functionality

In {{04-message-passing.pr}}, a colleague refactored the `chatbot::query_chat` function to become a method on a new `Chatbot` data structure. This method is also now stateful. Your task is to update the application server so it works with this new API.

### Performance

It is essential that you call `Chatbot::new` **once** in the entire lifetime of the application. That is, you should create a single `Chatbot` instance and have every call to the `/chat` route run queries against that instance.

Note: it is possible to implement parts of this problem using shared-memory concurrency primitives like `Mutex`, but you are strongly encouraged to instead practice with message passing (as described in the background).

## Background

### Non-`const` global variables 

If you need to declare a global variable involving a heap allocation, you can use the [`LazyLock`] primitive in the standard library. For example:

```rust
fn formula() -> &'static String {
    static SOLUTION: LazyLock<String> = LazyLock::new(|| format!("1 + 1 = {}", 1 + 1));
    &*SOLUTION
}
```

### Message passing

One way to perform asynchronous operations on shared mutable objects is to use **message passing**. [Just like in the standard library][rust-book-mp], you can send and receive messages between futures. One such primitive is a multiple-producer-single-consumer channel, found in [`tokio::sync::mpsc`]. For example, as shown in the [documentation][mpsc-doc]:

```rust
let (tx, mut rx) = tokio::sync::mpsc::channel(100);

tokio::spawn(async move {
  for i in 0..10 {
    if let Err(_) = tx.send(i).await {
      println!("receiver dropped");
      return;
    }
  }
});

while let Some(i) = rx.recv().await {
  println!("got = {}", i);
}
```

The [`mpsc::channel`] function creates a channel of a sender (`tx`) and receiver (`rx`). The sender uses `tx.send(msg)` to send messages to the receiver, which are accessed by `rx.recv()`. There are two main difference vs. [`std::sync::mpsc::channel`]:
1. Tokio's channel is *bounded*. Senders can only queue up to a maximum number of messages (the first argument to `channel`). The future returned by `tx.send(msg)` will be pending until the channel has space to queue the message.
2. Tokio's sender and receiver are non-blocking. Receivers can perform other asynchronous operations while waiting to receive a message.

### One-shot channels

When a receiver needs to "talk back" to its sender, a convenient utility is [`tokio::sync::oneshot::channel`]. A one-shot channel passes a single value before being consumed. For example, you can combine `mpsc` and `oneshot` channels to receive a message and respond to it:

```rust
let (tx, mut rx) = tokio::sync::mpsc::channel(100);

tokio::spawn(async move {
  loop {
    let (n, responder) = rx.recv().await.unwrap();
    responder.send(n + 1);
  }
});

let (tx2, rx2) = tokio::sync::oneshot::channel();
tx.send((1, tx2)).await.unwrap();
let n = rx2.await.unwrap();
assert_eq!(n, 2);
```

[`LazyLock`]: https://doc.rust-lang.org/stable/std/sync/struct.LazyLock.html
[rust-book-mp]: https://doc.rust-lang.org/book/ch16-02-message-passing.html
[`tokio::sync::mpsc`]: https://docs.rs/tokio/1.39.2/tokio/sync/mpsc/fn.channel.html
[mpsc-doc]: https://docs.rs/tokio/1.39.2/tokio/sync/mpsc/fn.channel.html#examples
[`mpsc::channel`]: https://docs.rs/tokio/1.39.2/tokio/sync/mpsc/fn.channel.html
[`std::sync::mpsc::channel`]: https://doc.rust-lang.org/nightly/std/sync/mpsc/fn.channel.html
[`tokio::sync::oneshot::channel`]: https://docs.rs/tokio/1.39.2/tokio/sync/oneshot/fn.channel.html
