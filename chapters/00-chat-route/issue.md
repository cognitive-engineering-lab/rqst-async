+++
title = "Add /chat route"
+++
Welcome to the RepoQuest tutorial on async/await in Rust! You're going to learn about async/await by doing hands-on exercises within a Rust codebase. This codebase is a web server that serves a basic chat app backed by a "large" "language" "model". 

Before we dive into async, you should first familiarize yourself with the codebase. This is a Rust monorepo with two crates: `miniserve`, a miniature HTTP server library, and `server`, the application using the library to serve the chatbot. You can see the documentation for `miniserve` by running:

```console
cargo doc -p miniserve --open
```

You can start the web server by running:

```console
cargo run
```

And then visit <http://localhost:3000> in your browser.

## Task

To get warmed up, we'll start with a non-async task. You may have noticed that the chat bar in the web app does nothing. Your task is to implement a naive chatbot on the `/chat` route of the server. Specifically, the web client will send a POST request containing a JSON data structure that looks like:

```json
{
  "messages": ["Hello, world!"]
}
```

The `/chat` route should return a JSON data structure with a new message appended to the list. For now, the message can be a fixed string, like this:

```json
{
  "messages": ["Hello world!", "And how does that make you feel?"]
}
```

If your implementation works, then you should be able to see your response appear in the frontend like this:

![ChatABC Frontend Screenshot]({{assets}}/chat-abc-frontend.png)

You should commit your changes to the `00-chat-route` branch, and merge the pull request {{00-chat-route.pr}} once you've completed the task. This will auto-close this issue, and you can proceed to the next task by running `repo-quest` in your terminal in the folder containing this repository.

## Background

Unlike a textbook, a RepoQuest tutorial is *task-first*. The learning process is oriented around solving concrete problems, and we give you those problems *before* giving you a bunch of explanations of concepts. We encourage you to read the background section in a demand-driven way &mdash; try solving the problem with what you know, and then come back and read the relevant sections of the background when you get stuck. Don't hesitate to use Rust Analyzer, Rustdoc, Google, StackOverflow, ChatGPT, or whatever other tools you find useful.

Instead, RepoQuest has one main rule for effective learning: **be curious.** Do not rush through the problems. Read the compiler's error messages. Try different ways of solving the same thing. Don't copy-paste or AI-generate solutions without understanding them. With that in mind, let's briefly cover some background.

### HTTP servers

This unit involves back-end web development. All the front-end work (HTML, CSS, JS) is done for you. You will be modifying an **HTTP server** (`server`) which uses an **HTTP server library** (`miniserve`). An HTTP server is a system that responds to **[HTTP requests]**. An HTTP request asks for a **method** to happen on a **route**. For our simple case, the method can be either `GET` (to retrieve static content like an HTML file) or `POST` (to send application data to the server, like a JSON blob). A route is the part of the URL that comes after the host, so <http://localhost:3000/chat> uses the `/chat` route of the HTTP server running on `localhost` port 3000.

### Miniserve

Miniserve is a simple HTTP server that is a thin layer on top of Rust's [`std::net`] module. It's a stand-in for more production-grade HTTP servers like [Axum] and [Rocket]. The starter code contains an example of how to create and run a miniserve server, and you can check the documentation / source code for more details.

### JSON

[JSON] is a string encoding of structured data. To deserialize and serialize JSON in Rust, you can use whatever library you're comfortable with. If you're not sure what to use, we recommend [Serde] and [serde_json]. Remember that a quick way to add a dependency is via `cargo add`, like this:

```console
cargo add serde -p server --features derive
cargo add serde_json -p server
```

### Debugging HTTP requests

If the web page isn't responding as you expect, try inspecting the response for the HTTP requests that's coming back from the server. You can do this using your browser's inspection tools. For example, in Google Chrome, go to View > Developer > Developer Tools and select the Network tab.


[HTTP requests]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Overview
[`std::net`]: https://doc.rust-lang.org/std/net/index.html
[Axum]: https://github.com/tokio-rs/axum
[Rocket]: https://rocket.rs/
[JSON]: https://developer.mozilla.org/en-US/docs/Learn/JavaScript/Objects/JSON
[Serde]: https://serde.rs/
[serde_json]: https://docs.rs/serde_json/latest/serde_json/
[close-issues]: https://github.blog/news-insights/product-news/closing-issues-via-commit-messages/
