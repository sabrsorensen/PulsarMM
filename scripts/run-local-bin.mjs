import { spawn, spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { setTimeout as delay } from 'node:timers/promises'

function resolveLocalBin(binName) {
  const suffix = process.platform === 'win32' ? '.cmd' : ''
  const localBin = path.join(process.cwd(), 'node_modules', '.bin', `${binName}${suffix}`)
  return fs.existsSync(localBin) ? localBin : binName
}

function drainPendingTtyInput() {
  if (process.platform === 'win32') {
    return
  }

  try {
    const nonBlocking = fs.constants.O_NONBLOCK ?? 0
    const fd = fs.openSync('/dev/tty', fs.constants.O_RDONLY | nonBlocking)
    const buffer = Buffer.alloc(1024)

    try {
      while (true) {
        try {
          const bytesRead = fs.readSync(fd, buffer, 0, buffer.length, null)
          if (bytesRead <= 0) {
            break
          }
        } catch (error) {
          if (error?.code === 'EAGAIN' || error?.code === 'EWOULDBLOCK') {
            break
          }
          break
        }
      }
    } finally {
      fs.closeSync(fd)
    }
  } catch {
    // Best-effort cleanup only.
  }
}

function stripTerminalQueries(text) {
  return text
    .replace(/\x1b\]1[01];\?(?:\x07|\x1b\\)/g, '')
    .replace(/\x1b\[6n/g, '')
    .replace(/\x1b\[(?:\?|>)?[0-9;]*c/g, '')
}

function forwardOutput(stream, writer) {
  if (!stream) {
    return
  }

  stream.on('data', (chunk) => {
    writer.write(stripTerminalQueries(chunk.toString()))
  })
}

function restoreTerminalState() {
  if (process.platform === 'win32') {
    return
  }

  try {
    spawnSync('stty', ['sane'], { stdio: ['inherit', 'ignore', 'ignore'] })
  } catch {
    // Best-effort cleanup only.
  }

  drainPendingTtyInput()
}

async function settleTerminalState() {
  restoreTerminalState()

  if (process.platform === 'win32') {
    return
  }

  for (let attempt = 0; attempt < 3; attempt += 1) {
    await delay(40)
    drainPendingTtyInput()
  }
}

const [, , binName, ...args] = process.argv

if (!binName) {
  console.error('Usage: node scripts/run-local-bin.mjs <bin> [args...]')
  process.exit(2)
}

const command = resolveLocalBin(binName)
const child = spawn(command, args, {
  env: process.env,
  stdio: ['ignore', 'pipe', 'pipe'],
  shell: false,
})

forwardOutput(child.stdout, process.stdout)
forwardOutput(child.stderr, process.stderr)

const forwardSignal = (signal) => {
  if (!child.killed) {
    child.kill(signal)
  }
}

process.on('SIGINT', () => forwardSignal('SIGINT'))
process.on('SIGTERM', () => forwardSignal('SIGTERM'))

child.on('error', async (error) => {
  await settleTerminalState()
  console.error(`Failed to start ${binName}:`, error)
  process.exit(1)
})

child.on('exit', async (code, signal) => {
  await settleTerminalState()

  if (signal) {
    process.kill(process.pid, signal)
    return
  }

  process.exit(code ?? 1)
})
