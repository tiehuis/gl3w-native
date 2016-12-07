gl3w-native
===========

A native implementation of [`gl3w`](https://github.com/skaslev/gl3w).

This is intended as a drop-in replacement by default. Some extra features are
added (such as [single-file generation](https://github.com/gingerBill/gl3w-Single-File))
and are available via flags.

Installation
============

### Compiling

This requires `rustc` version 1.13 or later.

```
git clone https://github.com/tiehuis/gl3w-native
cd gl3w-native
cargo build
```

Motivation
==========

*https://www.opengl.org/wiki/OpenGL_Loading_Library#GL3W*

>  ...as well as requiring that the user of GL3W have a Python installation.

> ...Unlike GL3W, this tool is written in Lua, which is downloadable for a variety of platforms (and has a much smaller install package than Python, if you care about that sort of thing). 

License
=======

`gl3w` is licensed under the public domain.
`glew-native` is licensed under the public domain.
