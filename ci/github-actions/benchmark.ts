import console from 'console'
import exec from 'exec-inline'
import process from 'process'
import shCmd from 'shell-escape'
import {
  SELF_BENCHMARK_MATRIX,
  SELF_BENCHMARK_HYPERFINE_NAMES,
  COMPETING_BENCHMARK_MATRIX,
  parseSelfBenchmarkCategory,
} from './benchmark/matrix'
import * as reportFiles from './benchmark/report-files'
import STRICT_BASH from './benchmark/strict-bash'

const pduTargets = process.argv.slice(2)
const errexit = (param: { readonly status: number | null }) => param.status !== 0

function section(title: string) {
  console.error(title)
  console.error('='.repeat(title.length))
  console.error()
}

section('Compare benchmark of pdu against other versions of itself')
for (const { category, units } of SELF_BENCHMARK_MATRIX) {
  const { commandSuffix, reportName } = parseSelfBenchmarkCategory(category)
  console.error({ category, commandSuffix, reportName })
  const unitNames = SELF_BENCHMARK_HYPERFINE_NAMES.map(name => `--command-name=${name}` as const)
  const pduCommands = units.map(unit => `${unit.pduExecName} ${commandSuffix.join(' ')} ${shCmd(pduTargets)}`)
  const exportReports = reportFiles.hyperfineArgs(reportName)
  const hyperfineCommand = shCmd(['hyperfine', '--warmup=3', ...exportReports, ...unitNames, ...pduCommands])
  const shellCommand = `${hyperfineCommand} 2>&1 | tee ${reportFiles.getFileName(reportName, 'log')}`
  exec(...STRICT_BASH, '-c', shellCommand).exit(errexit)
  console.error()
}

section('Compare benchmark of pdu against its competitors')
for (const { id, pduCliArgs, competitors } of COMPETING_BENCHMARK_MATRIX) {
  const commands = [
    `pdu ${pduCliArgs.join(' ')} tmp.sample`,
    ...competitors.map(argv => `${argv.join(' ')} tmp.sample` as const),
  ] as const
  console.error({ id, commands })
  const reportName = `competing.${id}` as const
  const exportReports = reportFiles.hyperfineArgs(reportName)
  const commandLog = reportFiles.getFileName(reportName, 'log')
  const hyperfineCommand = shCmd(['hyperfine', '--warmup=3', ...exportReports, ...commands])
  const shellCommand = `${hyperfineCommand} 2>&1 | tee ${commandLog}`
  exec(...STRICT_BASH, '-c', shellCommand).exit(errexit)
  console.error()
}
