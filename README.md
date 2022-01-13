This is a native Rust project, but also accommodates Python bindings in a way that allow the database and/or server to be run from Python.

It has three main components, `server`, `auth` and `data`. The core of this project is the `auth` component, which will be fully written in pure Rust. The `auth` component concerns the general endpoint processing, OAuth/OpenID logic and JWT logic.
