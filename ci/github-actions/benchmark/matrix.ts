import { getReleasedPduName } from './pdu-programs'

export const QUANTITY = ['len', 'blksize', 'blocks'] as const
export const MAX_DEPTH = ['1', '3', '5', '10'] as const
export const MIN_RATIO = ['0.1', '0.01', '0'] as const
export const PROGRESS = [false, true] as const
export const NO_SORT = [false, true] as const

export interface SelfBenchmarkTopic {
  readonly quantity: typeof QUANTITY[number]
  readonly maxDepth: typeof MAX_DEPTH[number]
  readonly minRatio: typeof MIN_RATIO[number]
  readonly progress: typeof PROGRESS[number]
  readonly noSort: typeof NO_SORT[number]
}

export const SELF_BENCHMARK_TOPICS: readonly SelfBenchmarkTopic[] = [{}]
  .flatMap(topic => QUANTITY.map(quantity => ({ ...topic, quantity })))
  .flatMap(topic => MAX_DEPTH.map(maxDepth => ({ ...topic, maxDepth })))
  .flatMap(topic => MIN_RATIO.map(minRatio => ({ ...topic, minRatio })))
  .flatMap(topic => PROGRESS.map(progress => ({ ...topic, progress })))
  .flatMap(topic => NO_SORT.map(noSort => ({ ...topic, noSort })))

export function parseSelfBenchmarkTopic(topic: SelfBenchmarkTopic) {
  const reportName = Object.values(topic).join('-')

  const commandSuffix = [
    `--quantity=${topic.quantity}`,
    `--max-depth=${topic.maxDepth}`,
    `--minimal-ratio=${topic.minRatio}`,
    ...topic.progress ? ['--progress'] as const : [] as const,
    ...topic.noSort ? ['--no-sort'] as const : [] as const,
  ] as const

  return { topic, reportName, commandSuffix }
}

export const RELEASED_PDU_VERSIONS = [] as const

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

export const SELF_BENCHMARK_MATRIX = SELF_BENCHMARK_TOPICS
  .map(topic => ({ topic, units: UNITS }))

export function getSelfBenchmarkHyperfineName<Version extends string>(
  unit: SelfBenchmarkUnitDev | SelfBenchmarkUnitReleased<Version>,
) {
  return unit.pduVersion ? unit.pduVersion : 'dev' as const
}

export const SELF_BENCHMARK_HYPERFINE_NAMES = UNITS.map(getSelfBenchmarkHyperfineName)
