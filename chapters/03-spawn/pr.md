+++
title = "Make chatbot::query_chat compute intensive"
+++
"Improvements" to our "language model" have required it to become more computationally intensive. This is represented by `tokio::time::sleep` (a non-blocking operation) changing into `std::thread::sleep` (a blocking operation).
