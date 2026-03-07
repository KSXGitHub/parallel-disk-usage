import console from 'console'
import exec from 'exec-inline'
import shCmd from 'shell-escape'
import * as reportFiles from './benchmark/report-files'
import STRICT_BASH from './benchmark/strict-bash'

const PDU_DUST_DUA_MATRIX = [
  {
    id: 'apparent-size',
    pduCliArgs: ['--quantity=apparent-size'],
    competitors: [
      ['dust', '--no-progress', '--apparent-size'],
      ['dua', '--count-hard-links', '--apparent-size'],
    ],
  },
  {
    id: 'block-size',
    pduCliArgs: ['--quantity=block-size'],
    competitors: [
      ['dust', '--no-progress'],
      ['dua', '--count-hard-links'],
    ],
  },
  {
    id: 'deduplicate-hardlinks',
    pduCliArgs: ['--deduplicate-hardlinks'],
    competitors: [
      ['dust', '--no-progress'],
      ['dua'],
    ],
  },
] as const

const errexit = (param: { readonly status: number | null }) => param.status !== 0

for (const { id, pduCliArgs, competitors } of PDU_DUST_DUA_MATRIX) {
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
