export const PREFIX = 'tmp.benchmark-report'

export type Prefix = typeof PREFIX

export const MAP = {
  asciidoc: 'adoc',
  csv: 'csv',
  json: 'json',
  markdown: 'md',
  log: 'log',
} as const

export type Format = keyof typeof MAP
export type Extension = typeof MAP[Format]

export const getFileName = <Name extends string, Ext extends Extension>(name: Name, ext: Ext) =>
  `${PREFIX}.${name}.${ext}` as const

const arg = <Name extends string, Fmt extends Format>(name: Name, fmt: Fmt) =>
  `--export-${fmt}=${getFileName(name, MAP[fmt])}` as const
export const hyperfineArgs = <Name extends string>(name: Name) => [
  arg(name, 'asciidoc'),
  arg(name, 'csv'),
  arg(name, 'json'),
  arg(name, 'markdown'),
]
