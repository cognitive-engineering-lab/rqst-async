+++
title = "Integrate logger into chat route"
+++
## Task

### Functionality

In {{08-abstraction.pr}}, your colleague has implemented the infrastructure for a `Logger` data structure that collects the messages provided by users, and intermittently writes them to disk. Your task is to integrate this logger into the `/chat` route. The application should have a single persistent `Logger`. Every time a user queries `/chat`, then the user's most recent message is added via `Logger::append`. Then `Logger::save` is called immediately after `append`.

You should implement the logger via a message-passing protocol in the same way you designed the chatbot. The challenge in this task is to reuse as much code between your logger and chatbot as possible. That is, you should design an abstraction that is generic over both use cases. This is fully open-ended &mdash; be creative! And we encourage you to compare with our reference solution at the end. Remember that you can see the reference solution by running `repo-quest` and selecting "View reference solution".

## Background

### Async in traits

Your abstraction will likely involve a trait that contains an asynchronous method. At the time of writing (August 2024), this is supported to varying degrees in the compiler as detailed in [this announcement][async-fn]. In brief, our recommendation is that you should avoid using `async fn` in traits, and instead prefer the `impl Trait` representation. For example:

```rust
trait MyInterface {
  fn call(&mut self) -> impl Future<Output = i32>;
}

struct Example;
impl MyInterface for Example {
  async fn call(&mut self) -> i32 { // you can still use async in the impl
    0
  }
}
```

[async-fn]: https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html
