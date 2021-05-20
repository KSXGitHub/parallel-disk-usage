import { getOctokit, context } from '@actions/github'
import console from 'console'
import { readFileSync } from 'fs'
import process from 'process'
import { SELF_BENCHMARK_CATEGORIES, SelfBenchmarkCategory, parseSelfBenchmarkCategory } from './benchmark/matrix'
import * as reportFiles from './benchmark/report-files'
import * as env from './lib/env'

const auth = env.load('GITHUB_TOKEN')
const commitInfo = `* ref: ${context.issue.owner}/${context.issue.repo}@${context.sha}`

function loadReport(category: SelfBenchmarkCategory, ext: reportFiles.Extension) {
  const { reportName } = parseSelfBenchmarkCategory(category)
  const filePath = reportFiles.getFileName(reportName, ext)
  return readFileSync(filePath, 'utf-8')
}

function rendered(category: SelfBenchmarkCategory) {
  return [
    '',
    loadReport(category, 'md').trim(),
    '',
  ].join('\n')
}

function codeBlock(category: SelfBenchmarkCategory, summary: string, lang: reportFiles.Format) {
  return [
    '<details><summary>',
    summary,
    '</summary>',
    '',
    '```' + lang,
    loadReport(category, reportFiles.MAP[lang]).trim(),
    '```',
    '',
    '</details>',
  ].join('\n')
}

function categoryReport(category: SelfBenchmarkCategory) {
  const { commandSuffix } = parseSelfBenchmarkCategory(category)

  return [
    '<details>',
    `<summary><strong>${commandSuffix.join(' ')}</strong></summary>`,
    '',
    rendered(category),
    codeBlock(category, 'Logs', 'log'),
    codeBlock(category, 'JSON', 'json'),
    '',
    '</details>',
  ].join('\n')
}

const overallReport = [
  '## Benchmark Reports',
  '',
  commitInfo,
  '',
  ...SELF_BENCHMARK_CATEGORIES.map(categoryReport),
].join('\n')

async function main() {
  const github = getOctokit(auth).rest

  // TODO: update created comment
  await github.issues.createComment({
    issue_number: context.issue.number,
    owner: context.repo.owner,
    repo: context.repo.repo,
    body: overallReport,
  })
}

main().catch(error => {
  console.error(error)
  throw process.exit(1)
})
