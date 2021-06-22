These manifests only for testing purposes.

#TODO

- adopt configs for prod (carefully)
- integrate vault
- move k8s-manifests to environment repo
- integrate with ArgoCD (carefully)

#NOT CLEAR
- how to generate secrets data? (leaved empty secrets for node02;node03;node04)

#HOWTO

> cd dev01

> kustomize build  | kubectl apply -f -


Don't hesitate to ask DevOps for any help!
