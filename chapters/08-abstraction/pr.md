+++
title = "Add Logger data structure"
+++
Defines a `chatbot::Logger` type that has two methods: `Logger::append` for adding a message, and `Logger::save` for intermittently writing to disk. You can construct a logger via `Logger::default()`.
