+++
title = "Add frontend support for cancellation"
+++
Adds a button to the frontend which allows users to cancel running computations. Specifically, when a user hits "Cancel", this posts the `/cancel` route. The `/chat` route is expected to return a `type` field of either `"Success"` or `"Cancelled"`.
