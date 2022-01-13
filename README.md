This is a native Rust project, but also accommodates Python bindings in a way that allow the database and/or server to be run from Python.

It has three main components, `server`, `auth` and `data`. The core of this project is the `auth` component, which will be fully written in pure Rust. The `auth` component concerns the general endpoint processing, OAuth/OpenID logic and JWT logic. However, for wider compatibility, the `server` and `data` components can be swapped out with similar code written in other languages, as long as they fulfil the API required by the `auth` component.

Official Python variants (not bindings, but full-featured non-Rust code) for both these components will be constructed, as well as Python _bindings_ for all components. This would, for example, allow you to run the server and database logic on FastAPI, while having the core security components in Rust. Furthermore, the `data` component will also have Rust bindings of the Python variant. 

|          | Rust code | Python bindings | Rust bindings | Python code |
|----------|-----------|-----------------|---------------|-------------|
| `server` | Primary   | Primary         | No            | Yes         |
| `auth`   | Primary   | Primary         | No            | No          |
| `data`   | Primary   | Primary         | Yes           | Yes         |