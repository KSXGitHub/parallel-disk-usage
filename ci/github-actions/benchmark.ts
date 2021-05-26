import console from 'console'
import process from 'process'
import shCmd from 'shell-escape'
import {
  SELF_BENCHMARK_MATRIX,
  SELF_BENCHMARK_HYPERFINE_NAMES,
  parseSelfBenchmarkCategory,
} from './benchmark/matrix.js'
import * as reportFiles from './benchmark/report-files.js'
import STRICT_BASH from './benchmark/strict-bash.js'
import exec from './lib/exec-inline.js'

const pduTargets = process.argv.slice(2)

for (const { category, units } of SELF_BENCHMARK_MATRIX) {
  const { commandSuffix, reportName } = parseSelfBenchmarkCategory(category)
  console.error({ category, commandSuffix, reportName })
  const unitNames = SELF_BENCHMARK_HYPERFINE_NAMES.map(name => `--command-name=${name}` as const)
  const pduCommands = units.map(unit => `${unit.pduExecName} ${commandSuffix.join(' ')} ${shCmd(pduTargets)}`)
  const exportReports = reportFiles.hyperfineArgs(reportName)
  const hyperfineCommand = shCmd(['hyperfine', '--warmup=3', ...exportReports, ...unitNames, ...pduCommands])
  const shellCommand = `${hyperfineCommand} 2>&1 | tee ${reportFiles.getFileName(reportName, 'log')}`
  exec(...STRICT_BASH, '-c', shellCommand).errexit()
  console.error()
}
