# MIT License
#
# Copyright (c) 2017-present Mario Juárez <mario@mjp.one> (http://www.mjp.one)
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
# See: https://github.com/mariomka/regex-benchmark/blob/master/python/benchmark.py
# Modifications Copyright (c) 2024 Spencer Phillip Young under MIT License
import argparse
import sys
from timeit import default_timer as timer

parser = argparse.ArgumentParser('benchmark.py')
parser.add_argument('filename')
parser.add_argument('library', choices=('re', 'regexrs', 'regex'), default='regexrs', nargs='?')

args = parser.parse_args()

if args.library == 'regexrs':
    import re_rs as re
elif args.library == 're':
    import re
elif args.library == 'regex':
    import regex as re
else:
    raise ValueError('Invalid library choice')


def measure(data, pattern):
    start_time = timer()

    regex = re.compile(pattern)
    matches = re.findall(regex, data)

    elapsed_time = timer() - start_time

    print(str(elapsed_time * 1e3) + ' - ' + str(len(matches)))


with open(sys.argv[1], encoding='utf-8') as file:
    data = file.read()

    # Email
    measure(data, r'[\w\.+-]+@[\w\.-]+\.[\w\.-]+')

    # URI
    measure(data, r'[\w]+://[^/\s?#]+[^\s?#]+(?:\?[^\s#]*)?(?:#[^\s]*)?')

    # IP
    measure(data, r'(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9])')
