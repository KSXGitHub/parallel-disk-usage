import console from 'console'
import process from 'process'
import collectRegressions from './benchmark/collect-regressions.js'
import { ACCEPTABLE_PERFORMANCE_REGRESSION } from './benchmark/matrix.js'

const regressionCollection = [...collectRegressions()]

if (!regressionCollection.length) {
  console.error('There are no performance regressions.')
  throw process.exit(0)
}

console.error(`error: There are ${regressionCollection.length} performance regressions.`)

for (const { category, regressions } of regressionCollection) {
  console.error(':: category', JSON.stringify(category))
  for (const { current, standard } of regressions) {
    const actual = current.mean / standard.mean
    const acceptance = ACCEPTABLE_PERFORMANCE_REGRESSION
    console.error(
      `-> ${current.command} / ${standard.command} = ${current.mean} / ${standard.mean} = ${actual} > ${acceptance}`,
    )
  }
}

throw process.exit(1)
