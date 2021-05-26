import { svg, renderToString } from '@popeindustries/lit-html-server'
import assert from 'assert'
import console from 'console'
import { readdirSync, readFileSync, writeFileSync } from 'fs'
import path from 'path'
import process from 'process'
import { Report, ReportUnit, assertReport } from './benchmark/report-files'

const xmlHeader = '<?xml version="1.0" encoding="utf-8" ?>'
const xmlns = 'http://www.w3.org/2000/svg'

const padding = 10
const labelWidth = 255
const barWidth = 512
const numberWidth = 127
const barHeight = 16

const backgroundColor = 'white'
const textColor = 'black'
const barColor = ['red', 'green', 'blue', 'cyan', 'magenta', 'yellow', 'grey'] as const

const getBarColor = (index: number) => barColor[index % barColor.length]

const fontFamily = 'monospace'

function renderReport(report: Report) {
  assert(report.results.length > 0, 'There must be at least 1 report')
  const viewBoxHeight = report.results.length * barHeight
  const values = report.results.map(unit => unit.mean)
  const maxValue = Math.max(...values)
  const viewBoxWidth = labelWidth + barWidth + 5 * padding + numberWidth
  const coords = report.results.map((unit, index) => ({ ...unit, index, y: index * barHeight }))
  const labels = coords.map(({ command, y }) =>
    svg`<text
      x=${padding}
      y=${y + barHeight}
      width=${labelWidth}
      height=${barHeight}
      fill=${textColor}
      font-family=${fontFamily}
    >${command}</text>`
  )
  const bars = coords.map(({ y, index, mean }) =>
    svg`<rect
      x=${padding + labelWidth + padding}
      y=${y}
      width=${Math.round(mean * barWidth / maxValue)}
      height=${barHeight}
      fill=${getBarColor(index)}
    />`
  )
  const numbers = coords.map(({ y, mean }) =>
    svg`<text
      x=${padding + labelWidth + padding + barWidth + padding}
      y=${y + barHeight}
      width=${numberWidth}
      height=${barHeight}
      fill=${textColor}
      font-family=${fontFamily}
    >${String(mean).slice(0, 6) + 's'}</text>`
  )
  return svg`<svg
    xmlns=${xmlns}
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
  for (const jsonFile of readdirSync('.')) {
    if (!jsonFile.startsWith('tmp.benchmark-report.') || !jsonFile.endsWith('.json')) continue
    const svgFile = jsonFile.replace(/\.json$/, '.svg')
    console.error(jsonFile, '→', svgFile)
    const report = JSON.parse(readFileSync(jsonFile, 'utf-8'))
    assertReport(report)
    const svgSuffix = await renderToString(renderReport(report))
    const svgFileContent = `${xmlHeader}\n${svgSuffix}`
    writeFileSync(svgFile, svgFileContent)
  }
}

main().catch(error => {
  console.error(error)
  throw process.exit(1)
})
