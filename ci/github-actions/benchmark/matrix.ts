import { getReleasedPduName } from './pdu-programs'

export const QUANTITY = ['len', 'blksize', 'blocks'] as const
export const MAX_DEPTH = ['1', '10'] as const
export const MIN_RATIO = ['0.01', '0'] as const
export const PROGRESS = [false, true] as const
export const NO_SORT = [false, true] as const

export interface SelfBenchmarkCategory {
  readonly quantity: typeof QUANTITY[number]
  readonly maxDepth: typeof MAX_DEPTH[number]
  readonly minRatio: typeof MIN_RATIO[number]
  readonly progress: typeof PROGRESS[number]
  readonly noSort: typeof NO_SORT[number]
}

export const SELF_BENCHMARK_CATEGORIES: readonly SelfBenchmarkCategory[] = [{}]
  .flatMap(category => QUANTITY.map(quantity => ({ ...category, quantity })))
  .flatMap(category => MAX_DEPTH.map(maxDepth => ({ ...category, maxDepth })))
  .flatMap(category => MIN_RATIO.map(minRatio => ({ ...category, minRatio })))
  .flatMap(category => PROGRESS.map(progress => ({ ...category, progress })))
  .flatMap(category => NO_SORT.map(noSort => ({ ...category, noSort })))

export function parseSelfBenchmarkCategory(category: SelfBenchmarkCategory) {
  const { quantity, maxDepth, minRatio, progress, noSort } = category

  const reportName =
    `quantity=${quantity},maxDepth=${maxDepth},minRatio=${minRatio},progress=${progress},noSort=${noSort}` as const

  const commandSuffix = [
    `--quantity=${quantity}`,
    `--max-depth=${maxDepth}`,
    `--min-ratio=${minRatio}`,
    ...progress ? ['--progress'] as const : [] as const,
    ...noSort ? ['--no-sort'] as const : [] as const,
  ] as const

  return { category, reportName, commandSuffix }
}

export const RELEASED_PDU_VERSIONS = [
  '0.0.0',
] as const

export const ACCEPTABLE_PERFORMANCE_REGRESSION = 1.1 // 10%

export interface SelfBenchmarkUnit {
  readonly pduVersion?: string
  readonly pduExecName: string
}

export interface SelfBenchmarkUnitDev extends SelfBenchmarkUnit {
  readonly pduVersion?: undefined
  readonly pduExecName: 'pdu'
}

const DEV_UNIT: SelfBenchmarkUnitDev = { pduExecName: 'pdu' }

export interface SelfBenchmarkUnitReleased<Version extends string> extends SelfBenchmarkUnit {
  readonly pduVersion: Version
  readonly pduExecName: `pdu-${Version}`
}

export function createSelfBenchmarkUnitReleased<Version extends string>(
  pduVersion: Version,
): SelfBenchmarkUnitReleased<Version> {
  return { pduVersion, pduExecName: getReleasedPduName(pduVersion) }
}

const RELEASED_UNITS = RELEASED_PDU_VERSIONS.map(createSelfBenchmarkUnitReleased)

const UNITS = [DEV_UNIT, ...RELEASED_UNITS]

export const SELF_BENCHMARK_MATRIX = SELF_BENCHMARK_CATEGORIES
  .map(category => ({ category, units: UNITS }))

export function getSelfBenchmarkHyperfineName<Version extends string>(
  unit: SelfBenchmarkUnitDev | SelfBenchmarkUnitReleased<Version>,
) {
  return unit.pduExecName
}

export const SELF_BENCHMARK_HYPERFINE_NAMES = UNITS.map(getSelfBenchmarkHyperfineName)

export interface CompetingBenchmarkCategory {
  readonly id: string
  readonly pduCliArgs: readonly string[]
  readonly competitors: ReadonlyArray<readonly [string, ...string[]]>
}

export const COMPETING_BENCHMARK_MATRIX: readonly CompetingBenchmarkCategory[] = [
  {
    id: 'len',
    pduCliArgs: ['--quantity=len'],
    competitors: [
      ['dust', '--apparent-size'],
      ['dutree'],
      ['dua', '--apparent-size'],
      ['ncdu', '-o', '/dev/stdout', '-0'],
      ['gdu', '--show-apparent-size', '--non-interactive', '--no-progress'],
      ['du', '--apparent-size'],
    ],
  },
  {
    id: 'blksize',
    pduCliArgs: ['--quantity=blksize'],
    competitors: [
      ['dust'],
      ['dutree', '--usage'],
      ['dua'],
      ['ncdu', '-o', '/dev/stdout', '-0'],
      ['gdu', '--non-interactive', '--no-progress'],
      ['du'],
    ],
  },
  {
    id: 'top-down',
    pduCliArgs: ['--top-down'],
    competitors: [
      ['dust', '--apparent-size', '--reverse'],
      ['dutree'],
    ],
  },
  {
    id: 'summary',
    pduCliArgs: ['--max-depth=1'],
    competitors: [
      ['dutree', '--summary'],
      ['dua', '--apparent-size'],
      ['ncdu', '-o', '/dev/null', '-0'],
      ['du', '--apparent-size', '--total'],
    ],
  },
  {
    id: 'extreme-details',
    pduCliArgs: ['--min-ratio=0'],
    competitors: [
      ['dutree'],
      ['ncdu', '-o', '/dev/stdout', '-0'],
      ['du', '--apparent-size'],
    ],
  },
  {
    id: 'no-sort',
    pduCliArgs: ['--no-sort'],
    competitors: [
      ['du', '--apparent-size'],
      ['dua', '--apparent-size'],
      ['ncdu', '-o', '/dev/stdout', '-0'],
      ['gdu', '--show-apparent-size', '--non-interactive', '--no-progress'],
    ],
  },
  {
    id: 'no-sort+summary',
    pduCliArgs: ['--no-sort', '--max-depth=1'],
    competitors: [
      ['dua', '--apparent-size'],
      ['ncdu', '-o', '/dev/null', '-0'],
      ['gdu', '--show-apparent-size', '--non-interactive', '--no-progress'],
      ['du', '--apparent-size', '--total'],
    ],
  },
  {
    id: 'progress',
    pduCliArgs: ['--progress'],
    competitors: [
      ['ncdu', '-o', '/dev/stdout', '-1'],
      ['gdu', '--show-apparent-size', '--non-interactive'],
    ],
  },
]
