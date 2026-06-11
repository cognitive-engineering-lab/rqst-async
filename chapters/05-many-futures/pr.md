+++
title = "Add support for retrieval-augmented generation"
+++
Makes two changes to the chatbot API:
1. Add the `retrieval_documents` methods which returns a vector of paths to retrieve from given the current conversation context.
2. Adds the `docs` parameter to `query_chat` which should take the content of the documents specified by `retrieval_documents`.
