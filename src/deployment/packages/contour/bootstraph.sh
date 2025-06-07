


# TODO make release unique
helm install contour oci://registry-1.docker.io/bitnamicharts/contour \
  -f /workspace/src/deploy/packages/contour/values.yaml
