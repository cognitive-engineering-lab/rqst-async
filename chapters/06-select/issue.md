+++
title = "Add backend support for cancellation"
+++
## Task

### Functionality

In {{06-select.pr}}, a colleague added support on the frontend for cancellation. A user can click a new "Cancel" button which will send a post request to the `/cancel` route. When this route is posted, then the server should cancel any `query_chat` computation that is currently running.

To handle the case of cancellation, the frontend now expects a new field in response from the `/chat` route. The response should have a `type` field that contains the string `"Success"` or `"Cancelled"`. If the `/cancel` route is called while `/chat` is waiting on `query_chat`, then the `/chat` route should return `{type: "Cancelled"}`. Otherwise, it should return a success message like:

```json
{
  "type": "Success",
  "messages": ["A message", "A response"]
}
```

### Performance

The server should cancel running chat computations as soon as `/cancel` is called. To simplify this problem, we have reverted `query_chat` from being compute-bound to being I/O-bound.

## Background

### Serializing enums

If you want to represent a JSON data structure that can have one of two schemas (and you're using Serde), then check out Serde's [internally-tagged enum representation][internal-tag].

### Selecting from different futures

The `join!` macro is used when you want to wait for *all* of multiple futures to complete. Conversely, the [`select!`] macro lets you wait for *one* of multiple futures to complete. For example, you could implement a simple 1-second timeout like this:

```rust
async fn long_op() -> i32 {
    tokio::time::sleep(/* ... */).await;
    0
}

async fn long_op_timeout() -> Option<i32> {
    let op_fut = long_op();
    let sleep_fut = tokio::time::sleep(Duration::from_secs(1));
    tokio::select! {
        n = op_fut => Some(n),  // if op_fut returns, then return its value
        _ = sleep_fut => None   // if the sleep_fut returns, we've timed out
    }
}
```

`select!` has a bit of a specialized syntax, described in [its docs][`select!`].

[internal-tag]: https://serde.rs/enum-representations.html#internally-tagged
[`select!`]: https://docs.rs/tokio/1.39.2/tokio/macro.select.html
