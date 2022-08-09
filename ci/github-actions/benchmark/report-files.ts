import console from 'console'
import { readFileSync } from 'fs'

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

const postProcessSchema = <Schema extends {}>(schema: Schema) => ({
  ...schema,
  additionalProperties: true,
})

export interface ReportUnit {
  command: string
  mean: number
  min: number
  max: number
}

export interface Report {
  results: ReportUnit[]
}

export function assertReport(data: unknown): asserts data is Report {
  if (typeof data !== 'object' || data === null) {
    console.error(data)
    throw new TypeError(`Data is not an object: ${data}`)
  }
  const { results } = data as { [_ in string]: unknown }
  if (!Array.isArray(results)) {
    console.error(data)
    throw new TypeError(`Property 'results' is not an array`)
  }
  for (const item of results) {
    if (typeof item !== 'object' || data === null) {
      console.error(item)
      throw new TypeError(`An item is not an object: ${item}`)
    }
    for (const name of ['command', 'mean', 'min', 'max'] as const) {
      if (name in item) {
        continue
      }
      console.error(item)
      throw new TypeError(`Property '${name}' does not exist in an item`)
    }
  }
}

export function loadByPath(path: string): Report {
  const json = readFileSync(path, 'utf-8')
  const data = JSON.parse(json)
  assertReport(data)
  return data
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
