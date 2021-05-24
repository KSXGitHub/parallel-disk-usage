import console from 'console'
import process from 'process'

export function load(name: string, fallback?: string) {
  const value = process.env[name]
  if (typeof value === 'string') return value
  if (typeof fallback === 'string') return fallback
  console.error(`error: Environment variable ${name} is empty.`)
  throw process.exit(1)
}
