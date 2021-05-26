import { SpawnSyncReturns, spawnSync } from 'child_process'
import process from 'process'
import shCmd from 'shell-escape'

export class Exec {
  readonly #cmd: string
  readonly #args: readonly string[]
  readonly #res: SpawnSyncReturns<Buffer>

  constructor(cmd: string, ...args: string[]) {
    this.#cmd = cmd
    this.#args = args
    this.#res = spawnSync(cmd, args, { stdio: 'inherit' })
  }

  public command() {
    return shCmd([this.#cmd, ...this.#args])
  }

  public errexit() {
    if (this.#res.signal) {
      throw new Error(`Command ${this.command()} was terminated with signal ${this.#res.signal}`)
    }

    if (this.#res.status === null) {
      throw new Error(`Command ${this.command()} exits with status null`)
    }

    if (this.#res.status) {
      process.exit(this.#res.status)
    }
  }
}

export function exec(cmd: string, ...args: string[]) {
  return new Exec(cmd, ...args)
}

export default exec
