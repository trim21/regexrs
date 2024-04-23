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
