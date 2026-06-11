+++
title = "Port server to async"
+++
## Task

In {{01-async-await.pr}}, the `miniserve` library was refactored to use Rust's async feature. Unfortunately this broke the build, because the new API is not backwards-compatible. Your task is to port the `server` crate so it has the same functionality as before, but such that it works with the new miniserve API. Commit your solution to {{01-async-await.pr}}.

## Background

Remember &mdash; try solving the problem *before* reading the background. What compilers errors do you get? Do they give any helpful advice? How far can you get before you get stuck? Don't forget to pull the merged PR before you begin.

### Futures

A **future** is a computation that eventually returns a value. Rust represents futures as types that implement the [`std::future::Future`][future-trait] trait. The `Future` trait has an associated type `Output` which indicates the kind of value returned by the future. For example, if a function returns `impl Future<Output = i32>`, then that means "this function returns a future which eventually outputs an `i32`." If you need a refresher on `impl Trait` syntax, check out the [Rust book chapter on traits][traits].

### `async` keyword

The easiest way to make a future is to use the **`async` keyword**. The `async` keyword can annotate a function:

```rust
async fn returns_a_future() -> i32 { 
  0
}
```

Or it can annotate block:

```rust
fn returns_a_future() -> impl Future<Output = i32> { 
  async { 0 }
}
```

These two pieces of code do the same thing.

### Future bounds

The `Future` trait commonly occurs as a trait bound in library functions that take futures as input, or return futures as outputs. For example:

```rust
fn i_want_a_future<F: Future>(_f: F) {}
fn i_want_an_async_fn<A: Future, B: Fn() -> A>(_f: B) {}

fn main() {
  let fut = async { println!("Hello world"); };
  i_want_a_future(fut);
  
  
  async fn fut_fn() { println!("Hello world"); }
  i_want_an_async_fn(fut_fn);
}
```

If you get a compiler error that says `Foo is not a future`, that means that `Foo` needs to implement the `Future` trait.

### `await` keyword

The easiest way to get the value out of a future is to use the **`await` keyword**. For example, you could await the output of `returns_a_future` like this:

```rust
async fn print_zero() {
  let zero = returns_a_future().await;
  println!("{zero}");
}
```

The `await` keyword can only be used in `async` functions or blocks. It basically means: "wait until this future has completed, and then get its value."

### Async in `main`

Rust does not allow the `main` function to be `async`. The reason has to do with how Rust futures are implemented, which we will discuss later. For now, the key thing to know is that async functions need to be executed in a **runtime**. An async runtime is code that runs futures and checks them for completion. Unlike most languages with async (e.g., Javascript, C#, etc.), Rust does not have a built-in async runtime. You have to provide one yourself.

### Providing an async runtime

For this tutorial, we will be using [Tokio], a popular and featureful async runtime (but the core concepts will translate to other runtimes). `miniserve` is already using Tokio. To use it within `server`, add it as a dependency using the `full` feature, e.g.,

```console
cargo add tokio --features full -p server
```

The simplest way to setup the Tokio runtime is to annotate the `main` function like this:

```rust
#[tokio::main]
async fn main() {
 // ...
}
```

<details>

<summary>How does <code>#[tokio::main]</code> work?</summary>

The macro isn't too complicated &mdash; try seeing what it expands to by running [`cargo expand`]. Basically, it moves the body of the main function into an async block, and then defines a new (non-async) main which passes that async block to a newly-created Tokio runtime.

</details>



[future-trait]: https://doc.rust-lang.org/std/future/trait.Future.html
[traits]: https://doc.rust-lang.org/book/ch10-02-traits.html
[Tokio]: https://tokio.rs/
[`cargo expand`]: https://github.com/dtolnay/cargo-expand
