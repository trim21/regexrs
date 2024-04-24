import pytest
import regexrs as re


def test_simple():
    pattern = re.compile(r'(\w+) (\w+)')
    match = pattern.match('hello rust')
    assert match.groups() == ('hello', 'rust')
    assert match.pos == 0
    assert match.endpos == 10


def test_flag_i():
    pattern = re.compile(r'hello rust', re.I)
    match = pattern.match('Hello Rust')
    assert match is not None


def test_findall_compiled():
    pattern = re.compile(r'\w+')
    assert re.findall(pattern, 'hello rust') == ['hello', 'rust']


def test_findall_compiled_pattern_with_flag_errors():
    pattern = re.compile(r'\w+')
    with pytest.raises(TypeError):
        re.findall(pattern, 'hello rust', re.I)


def test_findall_string():
    pattern = r'\w+'
    assert re.findall(pattern, 'hello rust') == ['hello', 'rust']
