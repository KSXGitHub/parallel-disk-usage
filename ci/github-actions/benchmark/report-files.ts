import { Type, Static } from '@sinclair/typebox'
import console from 'console'
import { readFileSync } from 'fs'
import process from 'process'
import createAjv from '../lib/ajv'

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

export const ReportUnit = Type.Object({
  command: Type.String(),
  mean: Type.Number(),
  min: Type.Number(),
  max: Type.Number(),
})
export type ReportUnit = Static<typeof ReportUnit>

export const Report = Type.Object({
  results: Type.Array(ReportUnit),
})
export type Report = Static<typeof Report>

export function loadByPath(path: string): Report {
  const json = readFileSync(path, 'utf-8')
  const data = JSON.parse(json)
  const ajv = createAjv()
  const valid = ajv.validate(Report, data)
  if (valid) return data as Report
  console.error('ValidationError', { data })
  console.error(ajv.errorsText(ajv.errors))
  throw process.exit(1)
}

export const isRegressed = (
  current: ReportUnit,
  standard: ReportUnit,
  maxRatio: number,
) => current.mean > standard.mean * maxRatio

export interface Regression {
  readonly current: ReportUnit
  readonly standard: ReportUnit
}

export function* detectRegressions(report: Report, maxRatio: number): Generator<Regression> {
  const [current, ...standards] = report.results
  if (!current) {
    throw new Error(`No reports`)
  }
  for (const standard of standards) {
    if (isRegressed(current, standard, maxRatio)) {
      yield { current, standard }
    }
  }
}
