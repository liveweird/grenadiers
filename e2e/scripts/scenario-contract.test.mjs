import assert from 'node:assert/strict'
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import test, { after } from 'node:test'

import { checkScenarioContract } from './scenario-contract.mjs'

const fixtureDirectories = []

after(async () => {
  await Promise.all(fixtureDirectories.map((directory) => rm(directory, { force: true, recursive: true })))
})

async function fixture(files) {
  const rootDirectory = await mkdtemp(join(tmpdir(), 'grenadiers-scenarios-'))
  fixtureDirectories.push(rootDirectory)
  await Promise.all(Object.entries(files).map(async ([path, contents]) => {
    const filename = join(rootDirectory, path)
    await mkdir(join(filename, '..'), { recursive: true })
    await writeFile(filename, contents)
  }))
  return rootDirectory
}

const importTest = "import { test } from '@playwright/test'\n"

test('accepts mapped static titles and ignores comments, helpers, hooks, and describes', async () => {
  const rootDirectory = await fixture({
    'scenarios/examples.md': "## Scenario: handles 'quoted' title\n## Scenario: static template\n",
    'tests/nested/examples.spec.ts': `${importTest}
      // test('comment title', () => {})
      const helper = "test('helper title', () => {})"
      test.beforeAll(() => {})
      test.describe('describe title', () => {
        test('handles \\'quoted\\' title', () => helper)
        test(\`static template\`, () => {})
      })`,
    'tests/nested/helper.ts': `${importTest}test('non-spec helper', () => {})`,
    'tests/build/generated.spec.ts': `${importTest}test('build output', () => {})`,
    'tests/node_modules/dependency.spec.ts': `${importTest}test('dependency output', () => {})`,
  })
  await assert.doesNotReject(checkScenarioContract({
    rootDirectory,
    mapping: { 'scenarios/examples.md': ['tests/nested/examples.spec.ts'] },
  }))
})

test('supports an aliased Playwright test import', async () => {
  const rootDirectory = await fixture({
    'scenarios/alias.md': '## Scenario: alias works\n',
    'alias-tests/alias.spec.ts': "import { test as scenario } from '@playwright/test'\nscenario.only('alias works', () => {})",
  })
  await assert.doesNotReject(checkScenarioContract({
    rootDirectory,
    mapping: { 'scenarios/alias.md': ['alias-tests/alias.spec.ts'] },
  }))
})

test('supports namespace imports, extended aliases, declaration modifiers, and conditional annotations', async () => {
  const rootDirectory = await fixture({
    'scenarios/forms.md': `## Scenario: namespace works
## Scenario: extended skip works
## Scenario: referenced only callback
## Scenario: referenced skip callback
## Scenario: referenced fixme callback
## Scenario: referenced fail callback
`,
    'form-tests/forms.spec.ts': `import * as pw from '@playwright/test'
      const scenario = pw.test.extend({})
      const body = () => {}
      pw.test('namespace works', () => {
        const enabled: boolean = Boolean(process.env.CI)
        const reason = 'referenced conditional reason'
        let typedReason: string
        const resolvedReason = getReason()
        pw.test.skip(true, 'conditional reason')
        pw.test.fixme(Boolean(process.env.CI), 'conditional reason')
        pw.test.fail(true, 'conditional reason')
        pw.test.skip(enabled, reason)
        pw.test.fixme(enabled, typedReason)
        pw.test.fail(enabled, reason)
        pw.test.skip(enabled, resolvedReason)
      })
      scenario.skip(\`extended skip works\`, () => {})
      scenario.only('referenced only callback', body)
      scenario.skip('referenced skip callback', body)
      scenario.fixme('referenced fixme callback', body)
      scenario.fail('referenced fail callback', body)`,
  })
  await assert.doesNotReject(checkScenarioContract({
    rootDirectory,
    mapping: { 'scenarios/forms.md': ['form-tests/forms.spec.ts'] },
  }))
})

test('rejects test imports from custom fixture modules explicitly', async () => {
  const rootDirectory = await fixture({
    'scenarios/custom.md': '## Scenario: custom fixture\n',
    'custom-tests/custom.spec.ts': "import { test } from './fixtures'\ntest('custom fixture', () => {})",
  })
  await assert.rejects(checkScenarioContract({
    rootDirectory,
    mapping: { 'scenarios/custom.md': ['custom-tests/custom.spec.ts'] },
  }), /imports test from unsupported module "\.\/fixtures"/)
})

test('reports duplicate scenario and test titles with every filename', async () => {
  const rootDirectory = await fixture({
    'scenarios/a.md': '## Scenario: repeated\n',
    'scenarios/b.md': '## Scenario: repeated\n',
    'a-tests/a.spec.ts': `${importTest}test('repeated', () => {})`,
    'b-tests/b.spec.ts': `${importTest}test('repeated', () => {})`,
  })
  await assert.rejects(checkScenarioContract({
    rootDirectory,
    mapping: {
      'scenarios/a.md': ['a-tests/a.spec.ts'],
      'scenarios/b.md': ['b-tests/b.spec.ts'],
    },
  }), (error) => {
    assert.match(error.message, /Scenario title "repeated" is duplicated in scenarios\/a\.md, scenarios\/b\.md/)
    assert.match(error.message, /Playwright test title "repeated" is duplicated in a-tests\/a\.spec\.ts, b-tests\/b\.spec\.ts/)
    return true
  })
})

test('rejects titles swapped between mapped files', async () => {
  const rootDirectory = await fixture({
    'scenarios/a.md': '## Scenario: title a\n',
    'scenarios/b.md': '## Scenario: title b\n',
    'a-tests/a.spec.ts': `${importTest}test('title b', () => {})`,
    'b-tests/b.spec.ts': `${importTest}test('title a', () => {})`,
  })
  await assert.rejects(checkScenarioContract({
    rootDirectory,
    mapping: {
      'scenarios/a.md': ['a-tests/a.spec.ts'],
      'scenarios/b.md': ['b-tests/b.spec.ts'],
    },
  }), /scenarios\/a\.md documents a title absent from its mapped specs/)
})

test('rejects unexpected and mapped-but-missing documents and specs', async () => {
  const rootDirectory = await fixture({
    'scenarios/unexpected.md': '## Scenario: extra\n',
    'extra-tests/unexpected.spec.ts': `${importTest}test('extra', () => {})`,
  })
  await assert.rejects(checkScenarioContract({
    rootDirectory,
    mapping: { 'scenarios/missing.md': ['tests/missing.spec.ts'] },
  }), (error) => {
    assert.match(error.message, /Unexpected scenario document: scenarios\/unexpected\.md/)
    assert.match(error.message, /Mapped scenario document does not exist: scenarios\/missing\.md/)
    assert.match(error.message, /Unexpected Playwright spec: extra-tests\/unexpected\.spec\.ts/)
    assert.match(error.message, /Mapped Playwright spec does not exist: tests\/missing\.spec\.ts/)
    return true
  })
})

test('rejects computed Playwright test titles with a source location', async () => {
  const rootDirectory = await fixture({
    'scenarios/dynamic.md': '## Scenario: dynamic title\n',
    'tests/dynamic.spec.ts': `${importTest}const suffix = 'title'\nconst body = () => {}\ntest.only(\`dynamic ${'${suffix}'}\`, body)`,
  })
  await assert.rejects(checkScenarioContract({
    rootDirectory,
    mapping: { 'scenarios/dynamic.md': ['tests/dynamic.spec.ts'] },
  }), /tests\/dynamic\.spec\.ts:4:1 has an unsupported dynamic Playwright test title/)
})

test('rejects ambiguous modifier overloads instead of omitting them', async () => {
  const rootDirectory = await fixture({
    'scenarios/ambiguous.md': '## Scenario: ambiguous title\n',
    'tests/ambiguous.spec.ts': `${importTest}test.skip(condition, reason)`,
  })
  await assert.rejects(checkScenarioContract({
    rootDirectory,
    mapping: { 'scenarios/ambiguous.md': ['tests/ambiguous.spec.ts'] },
  }), /tests\/ambiguous\.spec\.ts:2:1 has an ambiguous Playwright skip call/)
})

test('requires every spec in a grouped mapping to contribute a test declaration', async () => {
  const rootDirectory = await fixture({
    'scenarios/group.md': '## Scenario: covered by first spec\n',
    'group-tests/covered.spec.ts': `${importTest}test('covered by first spec', () => {})`,
    'group-tests/empty.spec.ts': `${importTest}// intentionally has no declaration\n`,
  })
  await assert.rejects(checkScenarioContract({
    rootDirectory,
    mapping: {
      'scenarios/group.md': ['group-tests/covered.spec.ts', 'group-tests/empty.spec.ts'],
    },
  }), /group-tests\/empty\.spec\.ts contains no recognized Playwright test declarations/)
})
