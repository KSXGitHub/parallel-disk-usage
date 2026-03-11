import { addPath } from '@actions/core'
import { ok, err } from '@tsfun/result'
import console from 'console'
import { ensureDir, createWriteStream, chmod } from 'fs-extra'
import fetch from 'node-fetch'
import path from 'path'
import process from 'process'
import { RELEASED_PDU_VERSIONS } from './benchmark/matrix'
import { getReleasedPduName, getReleasedPduMuslName } from './benchmark/pdu-programs'

const REPO = 'https://github.com/KSXGitHub/parallel-disk-usage'

type PduArtifactName = 'pdu-x86_64-unknown-linux-gnu' | 'pdu-x86_64-unknown-linux-musl'

async function downloadBinary(
  version: string,
  artifact: PduArtifactName,
  binaryPath: string,
) {
  const url = `${REPO}/releases/download/${version}/${artifact}` as const

  const responseResult = await fetch(url, {
    redirect: 'follow',
  }).then(ok, err)
  if (!responseResult.tag) {
    const { error } = responseResult
    return { step: 'fetch', version, url, error } as const
  }

  const response = responseResult.value
  if (!response.ok) {
    return { step: 'receive', version, url, response } as const
  }

  const readStream = response.body
  const writeStream = createWriteStream(binaryPath)
  const streamResult = await new Promise((resolve, reject) => {
    readStream.pipe(writeStream)
    readStream.on('error', reject)
    writeStream.on('finish', resolve)
  }).then(ok, err)
  if (!streamResult.tag) {
    const { error } = streamResult
    return { step: 'stream', version, binaryPath, readStream, writeStream, error } as const
  }

  const chmodResult = await chmod(binaryPath, 0o755).then(ok, err)
  if (!chmodResult.tag) {
    const { error } = chmodResult
    return { step: 'chmod', version, binaryPath, error } as const
  }

  return 'success' as const
}

async function main() {
  const targetDir = path.join(process.cwd(), 'RELEASED_PDU_VERSIONS.tmp')
  await ensureDir(targetDir)
  addPath(targetDir)

  const promises = RELEASED_PDU_VERSIONS.flatMap(version => [
    downloadBinary(
      version,
      'pdu-x86_64-unknown-linux-gnu',
      path.join(targetDir, getReleasedPduName(version)),
    ),
    downloadBinary(
      version,
      'pdu-x86_64-unknown-linux-musl',
      path.join(targetDir, getReleasedPduMuslName(version)),
    ),
  ])

  let errorCount = 0
  for await (const result of promises) {
    if (result === 'success') continue
    errorCount += 1
    console.error(result)
  }
  if (errorCount) {
    console.error(`${errorCount} errors occurred.`)
    throw process.exit(1)
  }
}

main().catch(error => {
  console.error(error)
  throw process.exit(1)
})
