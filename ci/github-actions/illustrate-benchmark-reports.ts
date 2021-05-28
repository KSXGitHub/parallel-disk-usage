import { svg, renderToString } from '@popeindustries/lit-html-server'
import assert from 'assert'
import console from 'console'
import { readdirSync, readFileSync, writeFileSync } from 'fs'
import process from 'process'
import { Report, assertReport } from './benchmark/report-files'

const xmlHeader = '<?xml version="1.0" encoding="utf-8" ?>'
const xmlns = 'http://www.w3.org/2000/svg'
const xlink = 'http://www.w3.org/1999/xlink'

const padding = 10
const charWidth = 9
const barColumnWidth = 512
const numberLength = 6
const numberColumnWidth = charWidth * numberLength
const rowHeight = 16

const backgroundColor = 'white'
const textColor = 'black'
const barColor = ['red', 'green', 'blue', 'cyan', 'magenta', 'yellow', 'grey'] as const

const getBarColor = (index: number) => barColor[index % barColor.length]

const fontFamily = 'Consolas, Menlo, monospace'

function renderReport(report: Report) {
  assert(report.results.length > 0, 'There must be at least 1 report')
  const viewBoxHeight = report.results.length * rowHeight
  const labelLengths = report.results.map(unit => unit.command.length)
  const labelColumnWidth = Math.max(...labelLengths) * charWidth
  const values = report.results.map(unit => unit.mean)
  const maxValue = Math.max(...values)
  const viewBoxWidth = labelColumnWidth + barColumnWidth + 5 * padding + numberColumnWidth
  const shapes = report.results.map((unit, index) => ({
    ...unit,
    index,
    labelX: padding,
    barX: padding + labelColumnWidth + padding,
    numberX: padding + labelColumnWidth + padding + barColumnWidth + padding,
    textY: (index + 0.8) * rowHeight,
    barY: index * rowHeight,
    labelWidth: unit.command.length * charWidth,
    barWidth: unit.mean * barColumnWidth / maxValue,
    numberContent: String(unit.mean).slice(0, numberLength - 1) + 's',
  }))
  const labels = shapes.map(({ command, labelX, textY, labelWidth }) =>
    svg`<text
      x=${labelX}
      y=${textY}
      width=${labelWidth}
      height=${rowHeight}
      textLength=${labelWidth}
      lengthAdjust="spacingAndGlyphs"
      fill=${textColor}
      font-family=${fontFamily}
    >${command}</text>`
  )
  const bars = shapes.map(({ index, barX, barY, barWidth }) =>
    svg`<rect
      x=${barX}
      y=${barY}
      width=${barWidth}
      height=${rowHeight}
      fill=${getBarColor(index)}
    />`
  )
  const numbers = shapes.map(({ numberX, textY, numberContent }) =>
    svg`<text
      x=${numberX}
      y=${textY}
      width=${numberColumnWidth}
      height=${rowHeight}
      fill=${textColor}
      font-family=${fontFamily}
    >${numberContent}</text>`
  )
  return svg`<svg
    xmlns=${xmlns}
    xmlns:xlink=${xlink}
    viewBox=${[0, 0, viewBoxWidth, viewBoxHeight].join(' ')}
  >
    <rect
      id="background"
      x="0"
      y="0"
      width=${viewBoxWidth}
      height=${viewBoxHeight}
      fill=${backgroundColor}
    />
    ${labels}
    ${bars}
    ${numbers}
  </svg>`
}

async function main() {
  const svgFiles = []
  for (const jsonFile of readdirSync('.')) {
    if (!jsonFile.startsWith('tmp.benchmark-report.') || !jsonFile.endsWith('.json')) continue
    const svgFile = jsonFile.replace(/\.json$/, '.svg')
    svgFiles.push(svgFile)
    console.error(jsonFile, '→', svgFile)
    const report = JSON.parse(readFileSync(jsonFile, 'utf-8'))
    assertReport(report)
    const svgSuffix = await renderToString(renderReport(report))
    const svgFileContent = `${xmlHeader}\n${svgSuffix}`
    writeFileSync(svgFile, svgFileContent)
  }
  const markdown = svgFiles
    .map(svgFile => `## ${svgFile}\n![${svgFile}](./${svgFile})\n`)
    .join('\n')
  writeFileSync('tmp.benchmark-report.CHARTS.md', `# Benchmark Charts\n\n${markdown}`)
}

main().catch(error => {
  console.error(error)
  throw process.exit(1)
})
