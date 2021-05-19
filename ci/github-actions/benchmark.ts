import exec from 'exec-inline'
import process from 'process'
import shCmd from 'shell-escape'
import { SELF_BENCHMARK_MATRIX, SELF_BENCHMARK_HYPERFINE_NAMES, parseSelfBenchmarkTopic } from './benchmark/matrix'
import * as reportFiles from './benchmark/report-files'
import STRICT_BASH from './benchmark/strict-bash'

const pduTargets = process.argv.slice(2)
const errexit = (param: { readonly status: number | null }) => param.status !== 0

for (const { topic, units } of SELF_BENCHMARK_MATRIX) {
  const { commandSuffix, reportName } = parseSelfBenchmarkTopic(topic)
  const unitNames = SELF_BENCHMARK_HYPERFINE_NAMES.map(name => `--command-name=${name}` as const)
  const pduCommands = units.map(unit => `${unit.pduExecName} ${commandSuffix.join(' ')} ${shCmd(pduTargets)}`)
  const exportReports = reportFiles.hyperfineArgs(reportName)
  const hyperfineCommand = shCmd(['hyperfine', '--warmup=3', ...exportReports, ...unitNames, ...pduCommands])
  const shellCommand = `${hyperfineCommand} | tee ${reportFiles.getFileName(reportName, 'log')}`
  exec(...STRICT_BASH, '-c', shellCommand).exit(errexit)
}
