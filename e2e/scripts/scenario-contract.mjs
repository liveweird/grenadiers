import { readFile, readdir } from 'node:fs/promises'
import { join, relative, sep } from 'node:path'

import ts from 'typescript'

const ignoredDirectories = new Set([
  '.git', '.playwright', 'build', 'coverage', 'dist', 'node_modules', 'out',
  'playwright-report', 'target', 'test-results',
])
const scenarioPattern = /^## Scenario: (.+)$/gm
const testDeclarationMethods = new Set(['fail', 'fixme', 'only', 'skip'])

function portablePath(path) {
  return path.split(sep).join('/')
}

async function walk(directory, accept) {
  const results = []
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (entry.isDirectory() && !ignoredDirectories.has(entry.name)) {
      results.push(...await walk(join(directory, entry.name), accept))
    } else if (entry.isFile() && accept(entry.name)) {
      results.push(join(directory, entry.name))
    }
  }
  return results
}

export async function discoverContractFiles(rootDirectory) {
  const entries = await readdir(rootDirectory, { withFileTypes: true })
  const testRoots = entries
    .filter((entry) => entry.isDirectory() && (entry.name === 'tests' || entry.name.endsWith('-tests')))
    .map((entry) => join(rootDirectory, entry.name))
  const documents = await walk(join(rootDirectory, 'scenarios'), (name) => name.endsWith('.md'))
  const specs = (await Promise.all(testRoots.map((directory) =>
    walk(directory, (name) => name.endsWith('.spec.ts'))))).flat()
  const relativeToRoot = (path) => portablePath(relative(rootDirectory, path))
  return {
    documents: documents.map(relativeToRoot).sort(),
    specs: specs.map(relativeToRoot).sort(),
  }
}

export function extractScenarioTitles(source, filename) {
  const titles = []
  for (const match of source.matchAll(scenarioPattern)) {
    titles.push({ title: match[1], filename })
  }
  return titles
}

function playwrightTestImports(sourceFile, filename) {
  const aliases = new Set()
  const namespaces = new Set()
  const otherNamespaces = new Map()
  for (const statement of sourceFile.statements) {
    if (!ts.isImportDeclaration(statement)
      || !ts.isStringLiteral(statement.moduleSpecifier)) continue
    const moduleName = statement.moduleSpecifier.text
    const bindings = statement.importClause?.namedBindings
    if (!bindings) continue
    if (ts.isNamespaceImport(bindings)) {
      if (moduleName === '@playwright/test') namespaces.add(bindings.name.text)
      else otherNamespaces.set(bindings.name.text, moduleName)
      continue
    }
    for (const element of bindings.elements) {
      if ((element.propertyName?.text ?? element.name.text) !== 'test') continue
      if (moduleName !== '@playwright/test') {
        throw new Error(`${filename}:${lineAndColumn(sourceFile, element)} imports test from unsupported module ${JSON.stringify(moduleName)}; import it directly from "@playwright/test"`)
      }
      aliases.add(element.name.text)
    }
  }
  return { aliases, namespaces, otherNamespaces }
}

function isTestReference(expression, imports) {
  if (ts.isIdentifier(expression) && imports.aliases.has(expression.text)) return true
  return ts.isPropertyAccessExpression(expression)
    && ts.isIdentifier(expression.expression)
    && imports.namespaces.has(expression.expression.text)
    && expression.name.text === 'test'
}

function testCallModifier(expression, imports) {
  if (isTestReference(expression, imports)) return null
  if (ts.isPropertyAccessExpression(expression)
    && isTestReference(expression.expression, imports)
    && testDeclarationMethods.has(expression.name.text)) return expression.name.text
  return undefined
}

function unsupportedNamespaceTest(expression, imports) {
  let candidate = expression
  if (ts.isPropertyAccessExpression(candidate)
    && testDeclarationMethods.has(candidate.name.text)) candidate = candidate.expression
  if (!ts.isPropertyAccessExpression(candidate)
    || !ts.isIdentifier(candidate.expression)
    || candidate.name.text !== 'test') return undefined
  return imports.otherNamespaces.get(candidate.expression.text)
}

function collectExtendedAliases(sourceFile, imports) {
  let changed = true
  while (changed) {
    changed = false
    const visit = (node) => {
      if (ts.isVariableDeclaration(node)
        && ts.isIdentifier(node.name)
        && node.initializer
        && ts.isCallExpression(node.initializer)
        && ts.isPropertyAccessExpression(node.initializer.expression)
        && node.initializer.expression.name.text === 'extend'
        && isTestReference(node.initializer.expression.expression, imports)
        && !imports.aliases.has(node.name.text)) {
        imports.aliases.add(node.name.text)
        changed = true
      }
      ts.forEachChild(node, visit)
    }
    visit(sourceFile)
  }
}

function lineAndColumn(sourceFile, node) {
  const position = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile))
  return `${position.line + 1}:${position.character + 1}`
}

function isStaticTitle(node) {
  return node && (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node))
}

function obviousInitializerKind(node) {
  if (!node) return undefined
  if (isStaticTitle(node)) return 'string'
  if (node.kind === ts.SyntaxKind.TrueKeyword || node.kind === ts.SyntaxKind.FalseKeyword) return 'boolean'
  if (ts.isArrowFunction(node) || ts.isFunctionExpression(node)) return 'callback'
  if (ts.isCallExpression(node)
    && ts.isIdentifier(node.expression)
    && node.expression.text === 'Boolean') return 'boolean'
  return undefined
}

function obviousTypeKind(node) {
  if (!node) return undefined
  if (node.kind === ts.SyntaxKind.StringKeyword) return 'string'
  if (node.kind === ts.SyntaxKind.BooleanKeyword) return 'boolean'
  if (ts.isFunctionTypeNode(node)) return 'callback'
  return undefined
}

function knownIdentifierKinds(sourceFile) {
  const kinds = new Map()
  const record = (name, kind) => {
    if (!kind) return
    const existing = kinds.get(name)
    kinds.set(name, existing && existing !== kind ? 'ambiguous' : kind)
  }
  const visit = (node) => {
    if ((ts.isVariableDeclaration(node) || ts.isParameter(node)) && ts.isIdentifier(node.name)) {
      record(node.name.text, obviousTypeKind(node.type) ?? obviousInitializerKind(node.initializer))
    } else if (ts.isFunctionDeclaration(node) && node.name) {
      record(node.name.text, 'callback')
    }
    ts.forEachChild(node, visit)
  }
  visit(sourceFile)
  return kinds
}

function expressionKind(node, knownKinds) {
  if (!node) return undefined
  if (ts.isIdentifier(node)) return knownKinds.get(node.text)
  return obviousInitializerKind(node)
}

function isModifierDeclaration(node, modifier, knownKinds, filename, sourceFile) {
  if (modifier === null || modifier === 'only') return true
  const [title, secondArgument] = node.arguments
  if (isStaticTitle(title)) return node.arguments.length > 1
  const secondKind = expressionKind(secondArgument, knownKinds)
  if (!secondArgument || secondKind === 'string') return false
  if (secondKind === 'callback') return true
  const conditionKind = expressionKind(title, knownKinds)
  if (conditionKind === 'boolean') return false
  throw new Error(`${filename}:${lineAndColumn(sourceFile, node)} has an ambiguous Playwright ${modifier} call; use a string literal or an identifier declared as string for an annotation reason`)
}

export function extractPlaywrightTitles(source, filename) {
  const sourceFile = ts.createSourceFile(filename, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS)
  const imports = playwrightTestImports(sourceFile, filename)
  collectExtendedAliases(sourceFile, imports)
  const knownKinds = knownIdentifierKinds(sourceFile)
  const titles = []
  const visit = (node) => {
    if (!ts.isCallExpression(node)) {
      ts.forEachChild(node, visit)
      return
    }
    const unsupportedModule = unsupportedNamespaceTest(node.expression, imports)
    if (unsupportedModule) {
      throw new Error(`${filename}:${lineAndColumn(sourceFile, node)} calls test from unsupported namespace module ${JSON.stringify(unsupportedModule)}; import it directly from "@playwright/test"`)
    }
    const modifier = testCallModifier(node.expression, imports)
    if (modifier !== undefined && isModifierDeclaration(node, modifier, knownKinds, filename, sourceFile)) {
      const title = node.arguments[0]
      if (!isStaticTitle(title)) {
        throw new Error(`${filename}:${lineAndColumn(sourceFile, node)} has an unsupported dynamic Playwright test title`)
      }
      titles.push({ title: title.text, filename })
    }
    ts.forEachChild(node, visit)
  }
  visit(sourceFile)
  return titles
}

function duplicates(entries, kind) {
  const byTitle = Map.groupBy(entries, (entry) => entry.title)
  return [...byTitle]
    .filter(([, occurrences]) => occurrences.length > 1)
    .map(([title, occurrences]) => `${kind} title ${JSON.stringify(title)} is duplicated in ${occurrences.map((entry) => entry.filename).join(', ')}`)
}

function inventoryErrors(discovered, mapping) {
  const mappedDocuments = Object.keys(mapping)
  const mappedSpecs = Object.values(mapping).flat()
  const compare = (actual, expected, kind) => [
    ...actual.filter((path) => !expected.includes(path)).map((path) => `Unexpected ${kind}: ${path}`),
    ...expected.filter((path) => !actual.includes(path)).map((path) => `Mapped ${kind} does not exist: ${path}`),
  ]
  const duplicateSpecs = [...Map.groupBy(mappedSpecs, (path) => path)]
    .filter(([, owners]) => owners.length > 1)
    .map(([path]) => `Spec is mapped more than once: ${path}`)
  return [
    ...compare(discovered.documents, mappedDocuments, 'scenario document'),
    ...compare(discovered.specs, mappedSpecs, 'Playwright spec'),
    ...duplicateSpecs,
  ]
}

function parityErrors(document, documented, tested) {
  const documentedTitles = new Set(documented.map((entry) => entry.title))
  const testedTitles = new Set(tested.map((entry) => entry.title))
  return [
    ...documented.filter((entry) => !testedTitles.has(entry.title))
      .map((entry) => `${entry.filename} documents a title absent from its mapped specs: ${JSON.stringify(entry.title)}`),
    ...tested.filter((entry) => !documentedTitles.has(entry.title))
      .map((entry) => `${entry.filename} has a title absent from ${document}: ${JSON.stringify(entry.title)}`),
  ]
}

export async function checkScenarioContract({ rootDirectory, mapping }) {
  const discovered = await discoverContractFiles(rootDirectory)
  const errors = inventoryErrors(discovered, mapping)
  const allDocumented = []
  const allTested = []

  if (errors.length === 0) {
    for (const [document, specs] of Object.entries(mapping)) {
      const documented = extractScenarioTitles(await readFile(join(rootDirectory, document), 'utf8'), document)
      const testedBySpec = await Promise.all(specs.map(async (spec) => ({
        spec,
        titles: extractPlaywrightTitles(await readFile(join(rootDirectory, spec), 'utf8'), spec),
      })))
      for (const { spec, titles } of testedBySpec) {
        if (titles.length === 0) errors.push(`${spec} contains no recognized Playwright test declarations`)
      }
      const tested = testedBySpec.flatMap(({ titles }) => titles)
      allDocumented.push(...documented)
      allTested.push(...tested)
      errors.push(...parityErrors(document, documented, tested))
    }
    errors.push(...duplicates(allDocumented, 'Scenario'), ...duplicates(allTested, 'Playwright test'))
  }

  if (errors.length > 0) throw new Error(`Scenario/test contract failed:\n- ${errors.join('\n- ')}`)
  return allDocumented.length
}
