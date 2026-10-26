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
  const namespace = requireSingle(resources, 'Namespace')
  for (const policy of ['enforce', 'audit', 'warn']) {
    if (namespace.metadata?.labels?.[`pod-security.kubernetes.io/${policy}`] !== 'restricted') {
      throw new Error(
        `Namespace ${targetName} must retain the restricted Pod Security ${policy} policy.`,
      )
    }
  }
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

function requireAppContainer(deployment) {
  const containers = deployment.spec?.template?.spec?.containers ?? []
  const matches = containers.filter((container) => container?.name === 'app')
  if (matches.length !== 1) {
    throw new Error(`Deployment ${targetName} must contain exactly one app container.`)
  }
  return matches[0]
}

function isPositiveResourceQuantity(value) {
  if (typeof value === 'number') {
    return Number.isFinite(value) && value > 0
  }
  if (typeof value !== 'string') {
    return false
  }

  // Kubernetes schema validation owns quantity syntax. This guard only establishes
  // that the supplied quantity has a positive numeric coefficient.
  const coefficient = value.match(/^\+?((?:\d+(?:\.\d*)?|\.\d+))/)?.[1]
  return coefficient !== undefined && Number(coefficient) > 0
}

function validateServiceAccountToken(podSpec) {
  if (podSpec?.automountServiceAccountToken !== false) {
    throw new Error(`Deployment ${targetName} must disable service-account token automounting.`)
  }
}

function validateEffectiveIdentity(podSecurity, containerSecurity) {
  const runAsNonRoot = containerSecurity?.runAsNonRoot ?? podSecurity?.runAsNonRoot
  const runAsUser = containerSecurity?.runAsUser ?? podSecurity?.runAsUser
  const runAsGroup = containerSecurity?.runAsGroup ?? podSecurity?.runAsGroup
  if (runAsNonRoot !== true) {
    throw new Error(`Deployment ${targetName} must require a non-root runtime identity.`)
  }
  if (runAsUser !== 65532 || runAsGroup !== 65532) {
    throw new Error(`Deployment ${targetName} must run with UID and GID 65532.`)
  }
}

function validateEffectiveSeccomp(podSecurity, containerSecurity) {
  const seccompType = containerSecurity?.seccompProfile?.type ?? podSecurity?.seccompProfile?.type
  if (seccompType !== 'RuntimeDefault') {
    throw new Error(`Deployment ${targetName} must use the RuntimeDefault seccomp profile.`)
  }
}

function validateEffectiveSecurity(deployment, app) {
  const podSpec = deployment.spec?.template?.spec
  const podSecurity = podSpec?.securityContext
  const containerSecurity = app.securityContext
  validateServiceAccountToken(podSpec)
  validateEffectiveIdentity(podSecurity, containerSecurity)
  validateEffectiveSeccomp(podSecurity, containerSecurity)
}

function validateContainerSecurity(app) {
  const containerSecurity = app.securityContext
  if (containerSecurity?.readOnlyRootFilesystem !== true) {
    throw new Error(`Deployment ${targetName} app must use a read-only root filesystem.`)
  }
  if (containerSecurity?.allowPrivilegeEscalation !== false) {
    throw new Error(`Deployment ${targetName} app must disable privilege escalation.`)
  }
  const droppedCapabilities = containerSecurity?.capabilities?.drop
  if (!Array.isArray(droppedCapabilities) || !droppedCapabilities.includes('ALL')) {
    throw new Error(`Deployment ${targetName} app must drop all Linux capabilities.`)
  }
  const addedCapabilities = containerSecurity?.capabilities?.add
  if (
    addedCapabilities !== undefined &&
    (!Array.isArray(addedCapabilities) || addedCapabilities.length !== 0)
  ) {
    throw new Error(`Deployment ${targetName} app must not add Linux capabilities.`)
  }
}

function validateResourceBudgets(app) {
  for (const budget of ['requests', 'limits']) {
    for (const resource of ['cpu', 'memory', 'ephemeral-storage']) {
      if (!isPositiveResourceQuantity(app.resources?.[budget]?.[resource])) {
        throw new Error(
          `Deployment ${targetName} app must define a positive ${resource} ${budget} quantity.`,
        )
      }
    }
  }
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
  const app = requireAppContainer(deployment)
  validateEffectiveSecurity(deployment, app)
  validateContainerSecurity(app)
  validateResourceBudgets(app)
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
