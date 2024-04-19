import regexrs as re


def test_simple():
    pattern = re.compile(r'(\w+) (\w+)')
    match = pattern.match('hello rust')
    assert match.groups() == ('hello', 'rust')
    assert match.pos == 0
    assert match.endpos == 10
