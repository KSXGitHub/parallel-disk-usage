import { getOctokit, context } from '@actions/github'
import console from 'console'
import { readFileSync } from 'fs'
import process from 'process'
import { SELF_BENCHMARK_TOPICS, SelfBenchmarkTopic, parseSelfBenchmarkTopic } from './benchmark/matrix'
import * as reportFiles from './benchmark/report-files'
import * as env from './lib/env'

const auth = env.load('GITHUB_TOKEN')
const commitInfo = `* ref: ${context.issue.owner}/${context.issue.repo}@${context.sha}`

function loadReport(topic: SelfBenchmarkTopic, ext: reportFiles.Extension) {
  const { reportName } = parseSelfBenchmarkTopic(topic)
  const filePath = reportFiles.getFileName(reportName, ext)
  return readFileSync(filePath, 'utf-8')
}

function rendered(topic: SelfBenchmarkTopic) {
  return [
    '<details><summary>Rendered</summary>',
    '',
    loadReport(topic, 'md'),
    '',
    '</details>',
  ].join('\n')
}

function codeBlock(topic: SelfBenchmarkTopic, summary: string, lang: reportFiles.Format) {
  return [
    '<details><summary>',
    summary,
    '</summary>',
    '',
    '```' + lang,
    loadReport(topic, reportFiles.MAP[lang]),
    '```',
    '',
    '</details>',
  ].join('\n')
}

function topicReport(topic: SelfBenchmarkTopic) {
  const { commandSuffix } = parseSelfBenchmarkTopic(topic)

  return [
    '<details><summary>',
    '',
    '### ' + commandSuffix.join(' '),
    '',
    '</summary>',
    '',
    rendered(topic),
    codeBlock(topic, 'Logs', 'log'),
    codeBlock(topic, 'JSON', 'json'),
    '',
    '</details>',
  ].join('\n')
}

const overallReport = [
  '## Benchmark Reports',
  '',
  commitInfo,
  '',
  ...SELF_BENCHMARK_TOPICS.map(topicReport),
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
