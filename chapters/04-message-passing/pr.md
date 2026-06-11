+++
title = "Add new Chatbot data structure"
+++
Refactors `chatbot::query_chat` into a method on the `Chatbot` data structure. This causes the build to break for the `server` crate. The main changes are:
* `Chatbot::new` takes as input a vector of emojis, which the chatbot will include to improve the attitude of users.
* `Chatbot::query_chat` is stateful because it rotates through emojis to keep the chat fresh.
