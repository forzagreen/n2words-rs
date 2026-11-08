#!/usr/bin/env node
/**
 * Imports the JavaScript n2words test fixtures into JSON files the Rust test
 * suite reads (tests/fixtures/{code}.json).
 *
 * Usage: node scripts/import-fixtures.mjs <path-to-js-n2words-repo> [outDir]
 *
 * Each JS fixture exports `cardinal`, `ordinal` and/or `currency` arrays of
 * `[input, expected, options?]`. Inputs are numbers, strings or bigints; a
 * bigint is encoded as `{ "bigint": "<digits>" }` because JSON has no bigint.
 * Option keys are converted from camelCase to snake_case (the Rust names).
 */
import { mkdirSync, readdirSync, writeFileSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { pathToFileURL } from 'node:url'

const [, , jsRepo, outDir = 'tests/fixtures'] = process.argv
if (!jsRepo) {
  console.error('usage: node scripts/import-fixtures.mjs <path-to-js-n2words-repo> [outDir]')
  process.exit(1)
}

const fixturesDir = resolve(jsRepo, 'test/fixtures')
mkdirSync(outDir, { recursive: true })

const snake = key => key.replace(/[A-Z]/g, c => '_' + c.toLowerCase())
const encodeInput = value => (typeof value === 'bigint' ? { bigint: value.toString() } : value)

let count = 0
for (const file of readdirSync(fixturesDir).filter(f => f.endsWith('.js')).sort()) {
  const code = file.slice(0, -3)
  const mod = await import(pathToFileURL(join(fixturesDir, file)).href)
  const out = { code }
  for (const form of ['cardinal', 'ordinal', 'currency']) {
    if (!mod[form]) continue
    out[form] = mod[form].map(([input, expected, options]) => {
      const entry = { input: encodeInput(input), expected }
      if (options) {
        entry.options = Object.fromEntries(Object.entries(options).map(([k, v]) => [snake(k), v]))
      }
      return entry
    })
  }
  writeFileSync(join(outDir, code + '.json'), JSON.stringify(out, null, 2) + '\n')
  count++
}
console.log(`wrote ${count} fixture files to ${outDir}`)
