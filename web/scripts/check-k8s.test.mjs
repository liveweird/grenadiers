import { describe, expect, it } from 'vitest'
import { validateRenderedK8s } from './check-k8s.mjs'

const namespace = `
apiVersion: v1
kind: Namespace
metadata:
  name: grenadiers
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
      containers:
        - name: app
          image: grenadiers-app:test
          ports:
            - name: http
              containerPort: 8085
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

describe('rendered Kubernetes guardrails', () => {
  it('accepts the valid Namespace, Deployment, and Service documents', () => {
    expect(() => validateRenderedK8s(rendered(namespace, deployment, service))).not.toThrow()
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
