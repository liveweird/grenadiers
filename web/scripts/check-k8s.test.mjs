import { describe, expect, it } from 'vitest'
import { validateRenderedK8s } from './check-k8s.mjs'

const namespace = `
apiVersion: v1
kind: Namespace
metadata:
  name: grenadiers
  labels:
    pod-security.kubernetes.io/enforce: restricted
    pod-security.kubernetes.io/audit: restricted
    pod-security.kubernetes.io/warn: restricted
`

const deployment = `
apiVersion: apps/v1
kind: Deployment
metadata:
  name: grenadiers
  namespace: grenadiers
spec:
  replicas: 1
  strategy:
    type: Recreate
  selector:
    matchLabels:
      app: grenadiers
  template:
    metadata:
      labels:
        app: grenadiers
    spec:
      automountServiceAccountToken: false
      securityContext:
        runAsNonRoot: true
        runAsUser: 65532
        runAsGroup: 65532
        seccompProfile:
          type: RuntimeDefault
      containers:
        - name: app
          image: grenadiers-app:test
          ports:
            - name: http
              containerPort: 8085
          resources:
            requests:
              cpu: 50m
              memory: 64Mi
              ephemeral-storage: 16Mi
            limits:
              cpu: 500m
              memory: 256Mi
              ephemeral-storage: 64Mi
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: true
            capabilities:
              drop:
                - ALL
`

const service = `
apiVersion: v1
kind: Service
metadata:
  name: grenadiers
  namespace: grenadiers
spec:
  selector:
    app: grenadiers
  ports:
    - port: 8085
      targetPort: http
`

function rendered(...resources) {
  return resources.join('\n---\n')
}

function withContainerSecurity(fields) {
  return deployment.replace(
    '          securityContext:\n',
    `          securityContext:\n${fields}`,
  )
}

describe('rendered Kubernetes guardrails', () => {
  it('accepts the valid Namespace, Deployment, and Service documents', () => {
    expect(() => validateRenderedK8s(rendered(namespace, deployment, service))).not.toThrow()
  })

  it('accepts alternate positive Kubernetes resource quantities', () => {
    const alternateQuantities = deployment
      .replace('cpu: 50m', 'cpu: 0.05')
      .replace('memory: 64Mi', 'memory: 65536Ki')
      .replace('ephemeral-storage: 64Mi', 'ephemeral-storage: 0.0625Gi')
    expect(() =>
      validateRenderedK8s(rendered(namespace, alternateQuantities, service)),
    ).not.toThrow()
  })

  it('accepts identical safe container security overrides and an empty capability add list', () => {
    const identicalOverrides = withContainerSecurity(
      '            runAsNonRoot: true\n' +
        '            runAsUser: 65532\n' +
        '            runAsGroup: 65532\n' +
        '            seccompProfile:\n' +
        '              type: RuntimeDefault\n',
    ).replace(
      '            capabilities:\n              drop:',
      '            capabilities:\n              add: []\n              drop:',
    )
    expect(() =>
      validateRenderedK8s(rendered(namespace, identicalOverrides, service)),
    ).not.toThrow()
  })

  it.each(['enforce', 'audit', 'warn'])(
    'rejects the Namespace without its restricted Pod Security %s label',
    (policy) => {
      const invalidNamespace = namespace.replace(
        `    pod-security.kubernetes.io/${policy}: restricted\n`,
        '',
      )
      expect(() => validateRenderedK8s(rendered(invalidNamespace, deployment, service))).toThrow(
        `restricted Pod Security ${policy} policy`,
      )
    },
  )

  it('rejects a weakened Namespace Pod Security policy', () => {
    const invalidNamespace = namespace.replace(
      'pod-security.kubernetes.io/enforce: restricted',
      'pod-security.kubernetes.io/enforce: privileged',
    )
    expect(() => validateRenderedK8s(rendered(invalidNamespace, deployment, service))).toThrow(
      'restricted Pod Security enforce policy',
    )
  })

  it.each([
    ['two replicas', deployment.replace('replicas: 1', 'replicas: 2'), 'exactly one replica'],
    [
      'RollingUpdate',
      deployment.replace('type: Recreate', 'type: RollingUpdate'),
      'Recreate strategy',
    ],
    [
      'the default strategy',
      deployment.replace('  strategy:\n    type: Recreate\n', ''),
      'Recreate strategy',
    ],
  ])('rejects a Deployment with %s', (_case, invalidDeployment, message) => {
    expect(() => validateRenderedK8s(rendered(namespace, invalidDeployment, service))).toThrow(message)
  })
})

describe('rendered Kubernetes security and resource guardrails', () => {
  it.each([
    [
      'service-account token automount omitted',
      deployment.replace('      automountServiceAccountToken: false\n', ''),
      'disable service-account token automounting',
    ],
    [
      'service-account token automount enabled',
      deployment.replace('automountServiceAccountToken: false', 'automountServiceAccountToken: true'),
      'disable service-account token automounting',
    ],
    [
      'runAsNonRoot omitted',
      deployment.replace('        runAsNonRoot: true\n', ''),
      'require a non-root runtime identity',
    ],
    [
      'runAsNonRoot disabled',
      deployment.replace('runAsNonRoot: true', 'runAsNonRoot: false'),
      'require a non-root runtime identity',
    ],
    [
      'runAsUser omitted',
      deployment.replace('        runAsUser: 65532\n', ''),
      'UID and GID 65532',
    ],
    [
      'root UID selected',
      deployment.replace('runAsUser: 65532', 'runAsUser: 0'),
      'UID and GID 65532',
    ],
    [
      'runAsGroup omitted',
      deployment.replace('        runAsGroup: 65532\n', ''),
      'UID and GID 65532',
    ],
    [
      'root GID selected',
      deployment.replace('runAsGroup: 65532', 'runAsGroup: 0'),
      'UID and GID 65532',
    ],
    [
      'seccomp type omitted',
      deployment.replace('          type: RuntimeDefault\n', ''),
      'RuntimeDefault seccomp profile',
    ],
    [
      'unconfined seccomp selected',
      deployment.replace('type: RuntimeDefault', 'type: Unconfined'),
      'RuntimeDefault seccomp profile',
    ],
    [
      'runAsNonRoot disabled by the app container',
      withContainerSecurity('            runAsNonRoot: false\n'),
      'require a non-root runtime identity',
    ],
    [
      'root UID selected by the app container',
      withContainerSecurity('            runAsUser: 0\n'),
      'UID and GID 65532',
    ],
    [
      'root GID selected by the app container',
      withContainerSecurity('            runAsGroup: 0\n'),
      'UID and GID 65532',
    ],
    [
      'unconfined seccomp selected by the app container',
      withContainerSecurity(
        '            seccompProfile:\n              type: Unconfined\n',
      ),
      'RuntimeDefault seccomp profile',
    ],
    [
      'read-only root filesystem omitted',
      deployment.replace('            readOnlyRootFilesystem: true\n', ''),
      'read-only root filesystem',
    ],
    [
      'writable root filesystem selected',
      deployment.replace('readOnlyRootFilesystem: true', 'readOnlyRootFilesystem: false'),
      'read-only root filesystem',
    ],
    [
      'privilege-escalation policy omitted',
      deployment.replace('            allowPrivilegeEscalation: false\n', ''),
      'disable privilege escalation',
    ],
    [
      'privilege escalation enabled',
      deployment.replace('allowPrivilegeEscalation: false', 'allowPrivilegeEscalation: true'),
      'disable privilege escalation',
    ],
    [
      'capability drop omitted',
      deployment.replace('                - ALL\n', ''),
      'drop all Linux capabilities',
    ],
    [
      'only a specific capability dropped',
      deployment.replace('- ALL', '- NET_RAW'),
      'drop all Linux capabilities',
    ],
    [
      'a capability added after dropping all capabilities',
      deployment.replace(
        '            capabilities:\n              drop:',
        '            capabilities:\n              add: [NET_BIND_SERVICE]\n              drop:',
      ),
      'must not add Linux capabilities',
    ],
  ])('rejects a Deployment with %s', (_case, invalidDeployment, message) => {
    expect(() => validateRenderedK8s(rendered(namespace, invalidDeployment, service))).toThrow(message)
  })

  it.each([
    ['CPU request omitted', '              cpu: 50m\n', '', 'positive cpu requests'],
    ['memory request set to zero', 'memory: 64Mi', 'memory: 0Mi', 'positive memory requests'],
    [
      'ephemeral-storage request omitted',
      '              ephemeral-storage: 16Mi\n',
      '',
      'positive ephemeral-storage requests',
    ],
    ['CPU limit set to zero', 'cpu: 500m', 'cpu: 0', 'positive cpu limits'],
    ['memory limit omitted', '              memory: 256Mi\n', '', 'positive memory limits'],
    [
      'ephemeral-storage limit set to zero',
      'ephemeral-storage: 64Mi',
      'ephemeral-storage: 0Mi',
      'positive ephemeral-storage limits',
    ],
  ])('rejects a Deployment with %s', (_case, target, replacement, message) => {
    const invalidDeployment = deployment.replace(target, replacement)
    expect(() => validateRenderedK8s(rendered(namespace, invalidDeployment, service))).toThrow(message)
  })
})

describe('rendered Kubernetes composition guardrails', () => {
  it('rejects an autoscaler targeting the Grenadiers Deployment', () => {
    const autoscaler = `
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: grenadiers
  namespace: grenadiers
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: grenadiers
`
    expect(() =>
      validateRenderedK8s(rendered(namespace, deployment, service, autoscaler)),
    ).toThrow('must not be targeted')
  })

  it('rejects duplicate YAML keys', () => {
    const duplicateKey = deployment.replace('  replicas: 1', '  replicas: 1\n  replicas: 2')
    expect(() => validateRenderedK8s(rendered(namespace, duplicateKey, service))).toThrow(
      'invalid YAML',
    )
  })

  it('rejects rendered output without the Grenadiers Deployment', () => {
    expect(() => validateRenderedK8s(rendered(namespace, service))).toThrow(
      'exactly one Deployment',
    )
  })

  it('rejects a Service selector that does not select the Deployment pods', () => {
    const mismatchedService = service.replace('app: grenadiers', 'app: another-app')
    expect(() => validateRenderedK8s(rendered(namespace, deployment, mismatchedService))).toThrow(
      'selector must match',
    )
  })

  it('rejects a named Service targetPort missing from the Deployment', () => {
    const invalidService = service.replace('targetPort: http', 'targetPort: metrics')
    expect(() => validateRenderedK8s(rendered(namespace, deployment, invalidService))).toThrow(
      'must name a Deployment container port',
    )
  })
})
