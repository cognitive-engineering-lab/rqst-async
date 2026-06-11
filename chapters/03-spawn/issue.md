+++
title = "Fix latency of compute-intensive chatbot"
+++
## Task

### Performance

In {{03-spawn.pr}}, a colleague modified the `chatbot::query_chat` function to become compute-intensive (blocking the thread it's called on). This unfortunately seems to have caused the latency of the `chat` route to increase to 4 seconds again. Your task is to fix this situation and reduce the latency of the route back to 2 seconds. 

**Bonus challenge:** Your solution should not require cloning any more data than was required before {{03-spawn.pr}}.

## Background

### Async futures are state machines

In Rust, futures using the `async` keyword compile to state machines. For example, if you have a simple async program like this:

```rust
async fn one() -> i32 { 0 }
async fn two() -> i32 { 0 }
async fn sum(a: i32) -> i32 {
    let b = one().await;
    let c = two().await;
    a + b + c
}
```

Roughly speaking, Rust will compile `sum` into a state machine by first generating an enum representing each state it can be in:

```rust
enum SumState {
    Initial { a: i32 },
    One     { a: i32,         state: OneState },
    Two     { a: i32, b: i32, state: TwoState }
}
```

The state of the future is its parameters (`a`), local variables (`b` and `c`) and the state of futures that it is recursively waiting on (`OneState` and `TwoState`). The future can be executed by polling it, like this:

```rust
impl SumState {
    fn poll(&mut self) -> Option<i32> {
        match self {
            SumState::Initial { a } => {
                *self = SumState::One { a: *a, state: OneState::new() };
                None
            }
            SumState::One { a, state } => match state.poll() {
                None => None,
                Some(b) => {
                    *self = SumState::Two { a: *a, b, state: TwoState::new() };
                    None
                }
            },
            SumState::Two { a, b, state } => match state.poll() {
                None => None,
                Some(c) => Some(*a + *b + c)
            }
        }
    }
}
```

Once you understand async futures as state machines, this reveals several important facts.
1. As first described in {{02-join.issue}}, the code in a future does not run until the future is polled.
2. Futures do not inherently run in parallel. Futures do not magically start executing on a new thread.
3. Futures "hold on" to their data, including parameters, inside an enum. This will have implications later for ownership!

### Spawning tasks

To run futures in parallel, you need to use an async runtime which is responsible for executing futures on different threads. Tokio provides such a runtime. You can tell Tokio to run a future on its thread pool by **spawning** a **task** with [`tokio::spawn`], like this:

```rust
async fn say_hello() {
  println!("Hello world!");
}

async fn run() {
  let handle: tokio::task::JoinHandle<()> = tokio::spawn(say_hello());
  let result: Result<(), tokio::task::JoinError> = handle.await;
  result.unwrap();
}
```

`tokio::spawn` is similar to [`std::thread::spawn`], except `tokio::spawn` takes a future as input rather than a function. It similarly returns a handle to wait for the completion of the spawned task. That handle returns `Result` for the possibility that the future panicked.

### Spawn safety

While some tools like `join!` can work with any future, other tools like `tokio::spawn` can only work with certain futures. We can understand those restrictions by reading the type signature of `spawn`:

```rust
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
  F: Future + Send + 'static,
  F::Output: Send + 'static
```

This signature says:
> `spawn` takes a value of type `F` where `F` is a `Future` that is also `Send` and `'static`. Additionally, the output of the future `F::Output` must also be `Send` and `'static`.

These restrictions are quite similar to the restrictions on closures passed to `std::thread::spawn`. A spawned future must be sendable across threads, so it cannot store a non-sendable type like `Rc`. A spawned future can also only store references with a `'static` lifetime, in case the spawned future outlives its initial context of creation.

You can reuse Rust's standard concurrency primitives to help in these situations. For example, [`Arc`] can share data across threads, and [`Mutex`] can mutate shared data. 

<details>

<summary>Is there an analogue of <a href="https://doc.rust-lang.org/stable/std/thread/fn.scope.html"><code>std::thread::scope</code></a>?</summary>

Not yet. Scoped tasks are still an open area of design for Rust async libraries. For a deep dive on this topic see: [A formulation for scoped tasks in Rust](https://tmandry.gitlab.io/blog/posts/2023-03-01-scoped-tasks/) and [The scoped task trilemma](https://without.boats/blog/the-scoped-task-trilemma/).

</details>

[`tokio::spawn`]: https://docs.rs/tokio/1.39.2/tokio/task/fn.spawn.html
[`std::thread::spawn`]: https://doc.rust-lang.org/std/thread/fn.spawn.html
[`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html
[`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
[`Mutex`]: https://doc.rust-lang.org/std/sync/struct.Mutex.html
