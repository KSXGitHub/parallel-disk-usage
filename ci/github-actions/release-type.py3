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

with open('Cargo.toml') as cargo_toml:
  data = toml.load(cargo_toml)
  version = dict_path(data, 'package', 'version')

  if version != release_tag:
    print(f'::warning ::RELEASE_TAG ({release_tag}) does not match Cargo.toml#package.version ({version})')
    print('::set-output name=release_type::none')
    print('::set-output name=is_release::false')
    print('::set-output name=is_prerelease::false')
    print(f'::set-output name=release_tag::{release_tag}')
    exit(0)

if re.match(r'^[0-9]+\.[0-9]+\.[0-9]+-.+$', release_tag):
  print('::set-output name=release_type::prerelease')
  print('::set-output name=is_release::true')
  print('::set-output name=is_prerelease::true')
  print(f'::set-output name=release_tag::{release_tag}')
  exit(0)

if re.match(r'^[0-9]+\.[0-9]+\.[0-9]+$', release_tag):
  print('::set-output name=release_type::official')
  print('::set-output name=is_release::true')
  print('::set-output name=is_prerelease::false')
  print(f'::set-output name=release_tag::{release_tag}')
  exit(0)

print('::set-output name=release_type::none')
print('::set-output name=is_release::false')
print('::set-output name=is_prerelease::false')
print(f'::set-output name=release_tag::{release_tag}')
exit(0)
