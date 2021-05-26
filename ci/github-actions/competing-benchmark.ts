import console from 'console'
import shCmd from 'shell-escape'
import { COMPETING_BENCHMARK_MATRIX } from './benchmark/matrix.js'
import * as reportFiles from './benchmark/report-files.js'
import STRICT_BASH from './benchmark/strict-bash.js'
import exec from './lib/exec-inline.js'

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
  exec(...STRICT_BASH, '-c', shellCommand).errexit()
  console.error()
}
