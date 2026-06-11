+++
title = "Integrate retrieval-augmented generation"
+++
## Task

### Functionality

In {{05-many-futures.pr}}, your colleague modified the `Chatbot` API to support *retrieval-augmented generation* (RAG), a cutting-edge chatbot technique. To use RAG, when a user provides a set of input messages, you should first call `Chatbot::retrieval_documents` to generate a set of paths to documents that the model thinks are relevant to the query. Then you should read those files from disk, and pass their contents in the new `docs` parameter of `Chatbot::query_chat`. 

### Performance

All documents should be retrieved from disk in parallel,

## Background

### Async I/O

Rust's standard I/O functions like [`std::fs::read_to_string`] are blocking, meaning they do not have optimal performance when used with async. Therefore, async libraries like Tokio provide utilities for asynchronous I/O such as the [`tokio::fs`] module. For example, you can read a file to a string using [`tokio::fs::read_to_string`] like this:

```rust
let contents = tokio::fs::read_to_string("some_file.txt").await.unwrap();
```

Note that at the time of writing, Tokio's async I/O is a simple wrapper over `std::fs` that launches I/O on a new thread to avoid blocking. See the [`tokio::fs`] module documentation for details.

### Waiting on many futures

As discussed in {{02-join.issue}}, if you need to wait on many futures, it's bad practice to `.await` in a sequence, and you should prefer to [`join!`] the futures instead. `join!` works well for a fixed number of futures, but you need different primitives for a variable number of futures.

There are a few different approaches depending on your goal. The most straightforward is to use a function like [`futures::future::join_all`] in the [`futures`] utility crate. For example:

```rust
async fn incr(n: usize) -> usize { n + 1 }
async fn compute() {
  let fut_iter = (0 .. 100).map(incr);
  let nums: Vec<usize> = futures::future::join_all(fut_iter).await;
  assert_eq!(&nums[..5], [1, 2, 3, 4, 5]);
}
```

Tokio provides a more complex data structure for this purpose, the [`JoinSet`]. For example:

```rust
async fn incr(n: usize) -> usize { n + 1 }
async fn compute() {
  let fut_iter = (0 .. 100).map(incr);
  let mut join_set = fut_iter.collect::<tokio::task::JoinSet<_>>();
  let mut nums = Vec::new();
  while let Some(result) = join_set.join_next().await {
    nums.push(result.unwrap());
  }
  // note that nums is unordered!
}
```

`JoinSet` is useful if you need more advanced capabilities like aborting tasks (which we will discuss further in a later chapter).

[`std::fs::read_to_string`]: https://doc.rust-lang.org/std/fs/fn.read_to_string.html
[`tokio::fs`]: https://docs.rs/tokio/1.39.2/tokio/fs/index.html
[`tokio::fs::read_to_string`]: https://docs.rs/tokio/1.39.2/tokio/fs/fn.read_to_string.html
[`join!`]: https://docs.rs/tokio/1.39.2/tokio/macro.join.html
[`futures::future::join_all`]: https://docs.rs/futures/0.3.30/futures/future/fn.join_all.html
[`futures`]: https://docs.rs/futures/0.3.30/futures/index.html
[`JoinSet`]: https://docs.rs/tokio/1.39.2/tokio/task/struct.JoinSet.html
