+++
title = "Add chatbot crate"
+++
This PR adds a new crate, `chatbot`, which provides functions for running an "intelligent" chatbot to respond to the user's queries. Those functions are:
* `chatbot::gen_random_number`: creates a random number from 0 to the maximum `usize`.
* `chatbot::query_chat`: given a set of input messages, queries the model for a vector of generated responses.
