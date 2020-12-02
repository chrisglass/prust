Prust
======

Prust is a simple example of an in-memory pastebin service.

It doesn't come with a lot of bells and whistles but does work as a very basic
pastebin service written in Rust.

It is mostly meant as a learning project, and to poke at the various web
frameworks available.


Running
--------

Simply running `cargo run` on the top-level directory should work on a
reasonable Rust version (this was developped on stable on Ubuntu 20.10)

The server is started on port 3000, to which you can point a web browser.

TODO
----

For now the basic functionality is there but there's a few things missing:

- Proper HTML
- Add some validation to the input (a paste should have a maximum size, so
  should the author field)
- Add some HTML escalping to the rendering back (right now it's vulnerable to
  code injection)
- Add a LRU-like invalidation to pastes (keeping e.g. 500 in memory).
- Add an upper memory limit for the LRU cache instead of number of entries.
- Add a Dockerfile
- Add a Helm chart to make k8s deployments trivial
