+++
title = "Port miniserve to async"
+++
Change the `miniserve` API to use Rust's async features via Tokio. Unfortunately, this breaks the `server` crate, so CI is failing. Help me finish the port.
