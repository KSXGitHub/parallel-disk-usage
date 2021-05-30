import { svg, renderToString } from '@popeindustries/lit-html-server'
import assert from 'assert'
import console from 'console'
import { readdirSync, readFileSync, writeFileSync } from 'fs'
import process from 'process'
import { Report, assertReport } from './benchmark/report-files'

const xmlHeader = '<?xml version="1.0" encoding="utf-8" ?>'
const xmlns = 'http://www.w3.org/2000/svg'
const xlink = 'http://www.w3.org/1999/xlink'

const hPadding = 10
const vPadding = 5
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
  const viewBoxHeight = report.results.length * rowHeight + 2 * vPadding
  const labelLengths = report.results.map(unit => unit.command.length)
  const labelColumnWidth = Math.max(...labelLengths) * charWidth
  const values = report.results.map(unit => unit.mean)
  const maxValue = Math.max(...values)
  const viewBoxWidth = labelColumnWidth + barColumnWidth + 5 * hPadding + numberColumnWidth
  const shapes = report.results
    .map((unit, index) => ({
      ...unit,
      index,
      name: unit.command.split(' ')[0].trim(),
      labelX: hPadding,
      barX: hPadding + labelColumnWidth + hPadding,
      numberX: hPadding + labelColumnWidth + hPadding + barColumnWidth + hPadding,
      textY: vPadding + (index + 0.8) * rowHeight,
      barY: vPadding + index * rowHeight,
      labelWidth: unit.command.length * charWidth,
      barWidth: unit.mean * barColumnWidth / maxValue,
      numberContent: String(unit.mean).slice(0, numberLength - 1) + 's',
    }))
    .map(shape => ({
      ...shape,
      barColor: getBarColor(shape.index),
      labelClassName: `label ${shape.name}`,
      barClassName: `bar ${shape.name}`,
      numberClassName: `number ${shape.name}`,
    }))
  const labels = shapes.map(shape =>
    svg`<text
      data-index=${shape.index}
      data-name=${shape.name}
      class=${shape.labelClassName}
      x=${shape.labelX}
      y=${shape.textY}
      width=${shape.labelWidth}
      height=${rowHeight}
      textLength=${shape.labelWidth}
      lengthAdjust="spacingAndGlyphs"
      fill=${textColor}
      font-family=${fontFamily}
    >${shape.command}</text>`
  )
  const bars = shapes.map(shape =>
    svg`<rect
      data-index=${shape.index}
      data-name=${shape.name}
      class=${shape.barClassName}
      x=${shape.barX}
      y=${shape.barY}
      width=${shape.barWidth}
      height=${rowHeight}
      fill=${shape.barColor}
    />`
  )
  const numbers = shapes.map(shape =>
    svg`<text
      data-index=${shape.index}
      data-name=${shape.name}
      data-value=${shape.mean}
      class=${shape.numberClassName}
      x=${shape.numberX}
      y=${shape.textY}
      width=${numberColumnWidth}
      height=${rowHeight}
      fill=${textColor}
      font-family=${fontFamily}
    >${shape.numberContent}</text>`
  )
  return svg`<svg
    xmlns=${xmlns}
    xmlns:xlink=${xlink}
    viewBox=${[0, 0, viewBoxWidth, viewBoxHeight].join(' ')}
  >
    <style>
      .label.pdu {
        font-weight: bold;
      }
    </style>
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
    report.results.sort((left, right) => left.mean - right.mean)
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
