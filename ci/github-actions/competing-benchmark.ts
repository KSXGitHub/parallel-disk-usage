import console from 'console'
import exec from 'exec-inline'
import shCmd from 'shell-escape'
import { COMPETING_BENCHMARK_MATRIX } from './benchmark/matrix'
import * as reportFiles from './benchmark/report-files'
import STRICT_BASH from './benchmark/strict-bash'

const errexit = (param: { readonly status: number | null }) => param.status !== 0

for (const { id, pduCliArgs, competitors } of COMPETING_BENCHMARK_MATRIX) {
  const commands = [
    `pdu ${pduCliArgs.join(' ')} tmp.sample`,
    ...competitors.map(argv => `${argv.join(' ')} tmp.sample` as const),
  ] as const
  console.error({ id, commands })
  const reportName = `competing.${id}` as const
  const exportReports = reportFiles.hyperfineArgs(reportName)
  const commandLog = reportFiles.getFileName(reportName, 'log')
  const hyperfineCommand = shCmd(['hyperfine', '--warmup=1', ...exportReports, ...commands])
  const shellCommand = `${hyperfineCommand} 2>&1 | tee ${commandLog}`
  exec(...STRICT_BASH, '-c', shellCommand).exit(errexit)
  console.error()
}
