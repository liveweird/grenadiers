import { readFile } from 'node:fs/promises'
import { pathToFileURL } from 'node:url'
import { parseAllDocuments } from 'yaml'

const targetName = 'grenadiers'

function objectsOfKind(resources, kind, name = targetName) {
  return resources.filter(
    (resource) => resource?.kind === kind && resource?.metadata?.name === name,
  )
}

function requireSingle(resources, kind) {
  const matches = objectsOfKind(resources, kind)
  if (matches.length !== 1) {
    throw new Error(`Rendered manifests must contain exactly one ${kind} named ${targetName}.`)
  }
  return matches[0]
}

function validateNamespace(resources) {
  requireSingle(resources, 'Namespace')
}

function validateDeployment(resources) {
  const deployment = requireSingle(resources, 'Deployment')
  if (deployment.metadata.namespace !== targetName) {
    throw new Error(`Deployment ${targetName} must be in namespace ${targetName}.`)
  }
  if (deployment.spec?.replicas !== 1) {
    throw new Error(`Deployment ${targetName} must render with exactly one replica.`)
  }
  if (deployment.spec?.strategy?.type !== 'Recreate') {
    throw new Error(`Deployment ${targetName} must use the Recreate strategy.`)
  }
  return deployment
}

function validateAutoscalers(resources) {
  const targetsGrenadiers = resources.some(
    (resource) =>
      resource?.kind === 'HorizontalPodAutoscaler' &&
      resource?.spec?.scaleTargetRef?.kind === 'Deployment' &&
      resource?.spec?.scaleTargetRef?.name === targetName,
  )
  if (targetsGrenadiers) {
    throw new Error(`Deployment ${targetName} must not be targeted by a HorizontalPodAutoscaler.`)
  }
}

function validateService(resources, deployment) {
  const service = requireSingle(resources, 'Service')
  if (service.metadata.namespace !== targetName) {
    throw new Error(`Service ${targetName} must be in namespace ${targetName}.`)
  }

  const selector = service.spec?.selector
  const podLabels = deployment.spec?.template?.metadata?.labels
  if (
    !selector ||
    Object.keys(selector).length === 0 ||
    Object.entries(selector).some(([key, value]) => podLabels?.[key] !== value)
  ) {
    throw new Error(`Service ${targetName} selector must match the Deployment pod labels.`)
  }

  const namedContainerPorts = new Set(
    (deployment.spec?.template?.spec?.containers ?? []).flatMap((container) =>
      (container.ports ?? []).map((port) => port.name).filter(Boolean),
    ),
  )
  const missingTarget = (service.spec?.ports ?? []).find(
    (port) => typeof port.targetPort === 'string' && !namedContainerPorts.has(port.targetPort),
  )
  if (missingTarget) {
    throw new Error(
      `Service ${targetName} targetPort ${missingTarget.targetPort} must name a Deployment container port.`,
    )
  }
}

export function validateRenderedK8s(source) {
  const documents = parseAllDocuments(source, { uniqueKeys: true })
  const parseErrors = documents.flatMap((document) => document.errors)
  if (parseErrors.length > 0) {
    throw new Error(`Rendered manifests are invalid YAML: ${parseErrors[0].message}`)
  }

  const resources = documents.map((document) => document.toJS()).filter(Boolean)
  validateNamespace(resources)
  const deployment = validateDeployment(resources)
  validateAutoscalers(resources)
  validateService(resources, deployment)
}

async function main() {
  const [manifestPath] = process.argv.slice(2)
  if (!manifestPath) {
    console.error('usage: node scripts/check-k8s.mjs <rendered-manifests.yaml>')
    process.exitCode = 2
    return
  }

  try {
    validateRenderedK8s(await readFile(manifestPath, 'utf8'))
    console.log('Rendered Kubernetes manifests preserve single-owner invariants.')
  } catch (error) {
    console.error(error instanceof Error ? error.message : error)
    process.exitCode = 1
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  await main()
}
