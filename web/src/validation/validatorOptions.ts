import type { Options } from 'ajv'

const defaultOptions = Object.freeze({ allErrors: true })
const currentOptions = new Map<string, Readonly<Options>>([
  ['gameplay-v18', Object.freeze({
    allErrors: true,
    inlineRefs: false,
    loopRequired: 10,
    loopEnum: 10,
  })],
])

/** Return a fresh canonical Ajv option object for a generated validator. */
export function validatorOptions(name: string): Options {
  return { ...(currentOptions.get(name) ?? defaultOptions) }
}
