import pathlib
import sys

import pytest

import re_rs
import re
import re2
import regex


Email = r'[\w\.+-]+@[\w\.-]+\.[\w\.-]+'
URI = r'[\w]+://[^/\s?#]+[^\s?#]+(?:\?[^\s#]*)?(?:#[^\s]*)?'
IP = r'(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9])'

lib_impl = {
    "re": re.compile,
    "re2": re2.compile,
    "regex": regex.compile,
    "regexrs": re_rs.compile,
}

data = pathlib.Path(r'C:\Users\Trim21\Downloads\input-text.txt').read_text(encoding='utf-8')


@pytest.mark.parametrize("impl", lib_impl.keys())
def test_email(benchmark, impl):
    benchmark(lib_impl[impl](Email).findall, data)


@pytest.mark.parametrize("impl", lib_impl.keys())
def test_url(benchmark, impl):
    benchmark(lib_impl[impl](URI).findall, data)


@pytest.mark.parametrize("impl", lib_impl.keys())
def test_ip(benchmark, impl):
    benchmark(lib_impl[impl](IP).findall, data)
