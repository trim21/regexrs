# regex-rust (regexrs)

Leverages the Rust [`regex` crate](https://crates.io/crates/regex) with PyO3 to create an interface similar to the Python
standard library `re` module.

```bash
pip install regex-rust
```

```python
>>> import regexrs as re
>>> pattern = re.compile(r'(\w+) (\w+)')
>>> m = pattern.match('hello rust')
>>> m.groups()
('hello', 'rust')
>>> m.pos
0
>>> m.endpos
10
```

## How to install from source

You can use `pip` to build and install.

```bash
pip install .
```

If you want to build manually:

```bash
pip install maturin
maturin build --release
```

## Status

Mostly incomplete and likely very buggy. I am using this mostly as an exercise in creating and distributing Python extensions using Rust and PyO3.
It's unclear if this will ever be a particularly useful project or not. If you're looking for a complete and performant
regex library for Python today, see the [regex project on PyPI](https://pypi.org/project/regex/).


Differences compared to standard lib:

- The `endpos` argument normally found in the `re` module is not supported in `regexrs` for the `match`/`search`/`findall`/`finditer` methods.
- Some regex features are not supported (because they are not supported by the `regex` crate), such as lookarounds and backreferences.
- Not all flags are supported. At present release, you may use the flags `IGNORECASE`, `MULTILINE`, `DOTALL` and `VERBOSE` (or their shorthand equivalents). These are translated to inline flags and prepended to your given pattern.
- Until a future release, there is no cache for avoiding re-compiling the same patterns multiple times
