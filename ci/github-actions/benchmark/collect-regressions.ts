import { SELF_BENCHMARK_CATEGORIES, ACCEPTABLE_PERFORMANCE_REGRESSION, parseSelfBenchmarkCategory } from './matrix'
import * as reportFiles from './report-files'

export function* collectRegressions(): Generator<readonly reportFiles.Regression[]> {
  for (const category of SELF_BENCHMARK_CATEGORIES) {
    const { reportName } = parseSelfBenchmarkCategory(category)
    const filePath = reportFiles.getFileName(reportName, 'json')
    const report = reportFiles.loadByPath(filePath)
    const regressions = [...reportFiles.detectRegressions(report, ACCEPTABLE_PERFORMANCE_REGRESSION)]
    if (regressions.length) yield regressions
  }
}

export default collectRegressions
