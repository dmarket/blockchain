These manifests only for testing purposes.

#TODO

- adopt configs for prod (carefully)
- integrate vault
- move k8s-manifests to environment repo
- integrate with ArgoCD (carefully)

#NOT CLEAR
- how to generate secrets data?

#HOWTO

> cd dev01

> kustomize build  | kubectl apply -f -


Don't hesitate to ask DevOps for any help!




kubectl annotate secret dockerhub vault.security.banzaicloud.io/vault-addr="https://vault.default.svc.cluster.local:8200"
kubectl annotate secret dockerhub vault.security.banzaicloud.io/vault-role="default"
kubectl annotate secret dockerhub vault.security.banzaicloud.io/vault-skip-verify="true"
kubectl annotate secret dockerhub vault.security.banzaicloud.io/vault-path="kubernetes"


kubectl annotate secret dockerhub-new vault.security.banzaicloud.io/vault-addr="http://10.157.0.109:8200"
kubectl annotate secret dockerhub-new vault.security.banzaicloud.io/vault-agent="true"
kubectl annotate secret dockerhub-new vault.security.banzaicloud.io/vault-auth-method="kubernetes"
kubectl annotate secret dockerhub-new vault.security.banzaicloud.io/vault-role="vault-auth"
kubectl annotate secret dockerhub-new vault.security.banzaicloud.io/vault-skip-verify="true"