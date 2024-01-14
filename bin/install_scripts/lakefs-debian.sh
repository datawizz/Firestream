
#!/bin/bash

VERSION=${VERSION:-"0.108.0"}
ARCH=$(uname -m)
DEST_DIR="/opt/lakefs"

if [ "$ARCH" == "x86_64" ]; then
  URL="https://github.com/treeverse/lakeFS/releases/download/v${VERSION}/lakeFS_${VERSION}_Linux_x86_64.tar.gz"
elif [ "$ARCH" == "aarch64" ]; then
  URL="https://github.com/treeverse/lakeFS/releases/download/v${VERSION}/lakeFS_${VERSION}_Linux_arm64.tar.gz"
else
  echo "Unsupported architecture"
  exit 1
fi

mkdir -p $DEST_DIR
curl -L "$URL" | tar xz -C $DEST_DIR
ln -sf $DEST_DIR/lakefs /usr/local/bin/lakefs

echo 'export PATH=$PATH:/usr/local/bin' >> ~/.bashrc
source ~/.bashrc

