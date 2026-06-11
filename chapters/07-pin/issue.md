+++
title = "Print out long-running chatbot tasks"
+++
## Task

### Functionality

In {{07-pin.pr}}, our colleague has been working on the performance of the `chatbot` crate functions. Now, these functions take a variable length of time between 0 and 5 seconds. Your task is to emit metrics that helps us understand when a given `/chat` call is taking too long. Specifically, after starting a `query_chat` call, then every second, you should print out the number of seconds that have elapsed. For instance, if a query takes 3.5 seconds, then you should print:

```
Waiting for 1 seconds
Waiting for 2 seconds
Waiting for 3 seconds
```

### Performance

The logging metrics should not be executed in a separate task from the `query_chat` call (i.e., do not use `tokio::spawn`).

## Background

### Reusing a future

Most operations, such as `join!` and `.await` consume ownership of a future. So this is not valid:

```rust
async fn test() {
    let fut = async { 0 };
    fut.await;
    let fut2 = fut; // error: use of moved value: `fut`
}
```

Sometimes you need to reuse a future, such as with `select!` where one or more futures may not have yet completed. To do so, you can await a *mutable reference* to the future, instead of the future itself. For example:

```rust
async fn test() {
    let mut fut = async { 0 };
    (&mut fut).await; // almost works...
    let fut2 = fut;
}
```

This almost works, except you will get an error:

```
error: `&mut {async block@...}` is not a future
  ...
  = help: the trait `Unpin` is not implemented for `&mut {async block@...}`
```

This error refers to a new concept:

### Pinning

#### Motivation for pinning
To understand pinning, we need to understand the problem it's solving. Consider a contrived future like this:

```rust
async fn one() -> i32 { 1 }
async fn example(a: i32) -> i32 {
    let b = &a;
    let c = one().await;
    *b + c
}
```

Recall from {{03-spawn.issue}} how the `example` function is lowered into a state machine. Its state machine would look something like:

```rust
enum ExampleState {
  Initial { a: i32 },
  One { a: i32, b: &i32, state: OneState }
}

impl ExampleState {
  fn poll(&mut self) -> Option<i32> {
    match self {
      ExampleState::Initial { a } => {
        *self = ExampleState::One { a, b: &a, state: OneState::new() };
        None
      }
      // ...
    }
  }
}
```
 
Except, this code doesn't work for two reasons. First, `ExampleState` has a reference `&i32`, so it needs to specify a lifetime for that reference. Second, `b: &a` is not valid, because that address becomes immediately invalid after calling `ExampleState::poll`.

In more general terms, the problem is that a future which contains a *local reference* (e.g., `b` referring to `a`) is a *self-referential struct*. Self-referential structs have two problems:
1. Rust's lifetime system does not allow you to express a struct where one field refers to another. Rust has a special solution for the case of async, which is an implementation detail that isn't important to cover further.
2. If a self-referential struct is moved, then its self-reference is invalidated, which would be memory-unsafe.

Therefore, Rust's design of futures lays out the following invariant: **once a future is polled, it must never be moved.** Enforcing this invariant is the problem solved by pinning.

#### The `Unpin` trait

Recall above we saw an error message that said:

```
  = help: the trait `Unpin` is not implemented for `&mut {async block@...}`
```

This refers to the marker trait [`Unpin`]. Being a marker trait (like `Send` and `Sync`) means that `Unpin` is automatically derived for all types where it is derivable. `Unpin` means "it is safe to move this object when you have ownership of it". Async futures do **not** implement `Unpin`, because they may have self-references.

#### The `Pin` type

To ensure that a future cannot move, we need to use the [`Pin`] type. Once a future is wrapped in `Pin` (i.e., the future is "pinned"), then we are allowed to poll it. This is enforced by the definition of the `Future` trait:

```rust
pub trait Future {
  type Output;
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

Unlike most methods in Rust APIs, `poll` can not be called on `&mut self` but rather `Pin<&mut Self>`. Note that `Pin` does not actually *do* anything to the pinned object, it merely serves to indicate the fact that an object is pinned.

To pin an object (i.e., to create a value of type `Pin<...>`), you have two options.
1. **Stack pinning:** the cheapest option is pin an object in-place on the stack using the [`pin!`] macro.
2. **Heap pinning:** a more expensive option is to pin an object into a new heap location using [`Box::pin`] function.

Stack pinning should be your default choice, but it means that a future can only live as long as its current scope. Heap pinning is useful for things like collecting a heterogeneous set of futures into a data structure.

#### Putting it all together

We can now fix up the previous example that refused to compile as follows:

```rust
use std::pin::pin;

async fn test() {
    let mut fut = pin!(async { 0 });
    (&mut fut).await;  // it works!
    let fut2 = fut;    // this works too!
}
```

[`Unpin`]: https://doc.rust-lang.org/std/marker/trait.Unpin.html
[`Pin`]: https://doc.rust-lang.org/std/pin/struct.Pin.html
[`pin!`]: https://doc.rust-lang.org/stable/std/pin/macro.pin.html
[`Box::pin`]: https://doc.rust-lang.org/stable/std/boxed/struct.Box.html#method.pin
