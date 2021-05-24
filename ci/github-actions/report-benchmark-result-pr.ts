import { getOctokit, context } from '@actions/github'
import console from 'console'
import { readFileSync } from 'fs'
import process from 'process'
import { Item as RegressionItem, collectRegressions } from './benchmark/collect-regressions'
import { SelfBenchmarkCategory, parseSelfBenchmarkCategory } from './benchmark/matrix'
import * as reportFiles from './benchmark/report-files'
import * as env from './lib/env'

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

function regressionReport(item: RegressionItem) {
  const { category } = item
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

async function main() {
  const commitInfo = `commit: ${context.issue.owner}/${context.issue.repo}@${context.sha}`
  const commentTitle = '## Benchmark Reports'

  const regressionCollection = [...collectRegressions()]
  const maxRegressionReports = 5
  const reportBody = regressionCollection.length
    ? regressionCollection.slice(0, maxRegressionReports).map(regressionReport).join('\n')
    : 'There are no regressions.'
  const overallReport = [
    commentTitle,
    '',
    commitInfo,
    '',
    reportBody,
  ].join('\n')

  const auth = env.load('GITHUB_TOKEN')
  const github = getOctokit(auth).rest

  const sharedOptions = {
    issue_number: context.issue.number,
    owner: context.repo.owner,
    repo: context.repo.repo,
  }

  const allComments = await github.issues.listComments(sharedOptions)
  const targetComments = allComments
    .data
    .filter(comment => comment.user?.login === 'github-actions[bot]')
    .filter(comment => comment.body?.split('\n').includes(commentTitle))

  if (!targetComments.length) {
    await github.issues.createComment({
      ...sharedOptions,
      body: overallReport,
    })
    return
  }

  await Promise.all(targetComments.map(comment =>
    github.issues.updateComment({
      ...sharedOptions,
      comment_id: comment.id,
      body: overallReport,
    })
  ))
}

main().catch(error => {
  console.error(error)
  throw process.exit(1)
})
