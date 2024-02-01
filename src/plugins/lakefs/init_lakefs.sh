



# LakeFS assumes that users will create the initial account (admnin) using the User Interface.
# There is support for creating the user using the CLI but it assumes that the real S3 is the endpoint, not Minio.
# The LakeFS code calls AWS to get the account ID, which fails when using Minio.

# 2023-09-02 Posted to LakeFS slack channel:
# https://lakefs.slack.com/archives/C016726JLJW/p1693677122692919




export POD_NAME=$(kubectl get pods --namespace default -l "app.kubernetes.io/instance=lakefs" -o jsonpath="{.items[0].metadata.name}")

kubectl exec -it $POD_NAME -- sh

lakefs setup --user-name fireworks --access-key-id TODO_CHANGE_ME2 --secret-access-key THIS_IS_A_SECRET_TODO_CHANGE_ME2