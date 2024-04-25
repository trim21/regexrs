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
>>> re.findall(r'\w+', 'hello rust')
['hello', 'rust']
>>> re.fullmatch(r'\w+', 'foo')
<regexrs.Match object; span=(0, 3), match="foo">
```

## Benchmarks

`benchmark.py` is largely borrowed from the [regex-benchmark](https://github.com/mariomka/regex-benchmark) project. You are expected to pass in a path to the file of the [input-text.txt file](https://github.com/mariomka/regex-benchmark/blob/master/input-text.txt) to `benchmark.py`.

This simple benchmark suggests that `regexrs` may be significantly faster than the `re` module from the standard library, at least in some use cases. Keep in mind that
this benchmark tests just three simple use cases on a single large text input and, therefore, its performance insights are quite limited.

Results as tested on Windows AMD64 Python 3.12.2 using pgo-optimized build - times in ms (lower is better):

| test  | regexrs   | re (stdlib) | [regex](https://pypi.org/project/regex/) | Compared to re    |
|-------|-----------|-------------|------------------------------------------|-------------------|
| Email | 12.51     | 354.53      | 690.15                                   | **28.34x faster** |
| URI   | 4.82      | 282.69      | 430.26                                   | **58.65x faster** |
| IP    | 4.71      | 321.37      | 25.43                                    | **68.23x faster** |

To run the benchmarks yourself:

```bash
# be sure to have run `pip install regex-rust` first
# to test regexrs:
python benchmark.py /path/to/input-text.txt

# to test stdlib re:
python benchmark.py /path/to/input-text.txt re

# be sure to have run `pip install regex` first
# to test regex library:
python benchmark.py /path/to/input-text.txt regex
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
