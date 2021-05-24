import {
  SELF_BENCHMARK_CATEGORIES,
  ACCEPTABLE_PERFORMANCE_REGRESSION,
  SelfBenchmarkCategory,
  parseSelfBenchmarkCategory,
} from './matrix'
import * as reportFiles from './report-files'

export interface Item {
  readonly category: SelfBenchmarkCategory
  readonly regressions: readonly reportFiles.Regression[]
}

export function* collectRegressions(): Generator<Item> {
  for (const category of SELF_BENCHMARK_CATEGORIES) {
    const { reportName } = parseSelfBenchmarkCategory(category)
    const filePath = reportFiles.getFileName(reportName, 'json')
    const report = reportFiles.loadByPath(filePath)
    const regressions = [...reportFiles.detectRegressions(report, ACCEPTABLE_PERFORMANCE_REGRESSION)]
    if (regressions.length) yield { category, regressions }
  }
}

export default collectRegressions
