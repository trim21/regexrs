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


def test_match_does_not_match():
    pattern = re.compile('foo(?P<name>bar)')
    assert pattern.match('123 foobar') is None


def test_match_at_position():
    pattern = re.compile('foo(?P<name>bar)')
    assert pattern.match('123 foobar', pos=4) is not None


def test_match_fn_does_not_match():
    assert re.match(r'foo(?P<name>bar)', '123 foobar') is None


def test_match_fn():
    assert re.match(r'foo(?P<name>bar)', 'foobar') is not None
