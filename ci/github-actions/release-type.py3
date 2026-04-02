#! /usr/bin/env python3
from os import environ
import re
import toml

release_tag = environ.get('RELEASE_TAG', None)

if not release_tag:
  print('::error ::Environment variable RELEASE_TAG is required but missing')
  exit(1)

tag_prefix = 'refs/tags/'
if release_tag.startswith(tag_prefix):
  release_tag = release_tag.replace(tag_prefix, '', 1)

def dict_path(data, head: str, *tail: str):
  if type(data) != dict: raise ValueError('Not a dict', data)
  value = data.get(head)
  if not tail: return value
  return dict_path(value, *tail)

def set_output(name: str, value: str):
  with open(environ['GITHUB_OUTPUT'], 'a') as fh:
    print(f'{name}={value}', file=fh)

with open('Cargo.toml') as cargo_toml:
  data = toml.load(cargo_toml)
  version = dict_path(data, 'package', 'version')

  if version != release_tag:
    print(f'::warning ::RELEASE_TAG ({release_tag}) does not match Cargo.toml#package.version ({version})')
    set_output('release_type', 'none')
    set_output('is_release', 'false')
    set_output('is_prerelease', 'false')
    set_output('release_tag', release_tag)
    exit(0)

if re.match(r'^[0-9]+\.[0-9]+\.[0-9]+-.+$', release_tag):
  set_output('release_type', 'prerelease')
  set_output('is_release', 'true')
  set_output('is_prerelease', 'true')
  set_output('release_tag', release_tag)
  exit(0)

if re.match(r'^[0-9]+\.[0-9]+\.[0-9]+$', release_tag):
  set_output('release_type', 'official')
  set_output('is_release', 'true')
  set_output('is_prerelease', 'false')
  set_output('release_tag', release_tag)
  exit(0)

set_output('release_type', 'none')
set_output('is_release', 'false')
set_output('is_prerelease', 'false')
set_output('release_tag', release_tag)
exit(0)
