import console from 'console'
import exec from 'exec-inline'
import process from 'process'
import shCmd from 'shell-escape'
import { GNU_VS_MUSL_BENCHMARK_MATRIX } from './benchmark/matrix'
import * as reportFiles from './benchmark/report-files'
import STRICT_BASH from './benchmark/strict-bash'

const pduTargets = process.argv.slice(2)
const errexit = (param: { readonly status: number | null }) => param.status !== 0

for (const { version, gnuExecName, muslExecName } of GNU_VS_MUSL_BENCHMARK_MATRIX) {
  const reportName = `gnu-vs-musl.${version}` as const
  console.error({ version, gnuExecName, muslExecName, reportName })
  const gnuCommand = `${gnuExecName} --threads=max ${shCmd(pduTargets)}`
  const muslCommand = `${muslExecName} --threads=max ${shCmd(pduTargets)}`
  const exportReports = reportFiles.hyperfineArgs(reportName)
  const hyperfineCommand = shCmd([
    'hyperfine',
    '--warmup=3',
    `--command-name=gnu`,
    `--command-name=musl`,
    ...exportReports,
    gnuCommand,
    muslCommand,
  ])
  const shellCommand = `${hyperfineCommand} 2>&1 | tee ${reportFiles.getFileName(reportName, 'log')}`
  exec(...STRICT_BASH, '-c', shellCommand).exit(errexit)
  console.error()
}
