#!/usr/bin/env bash
set -euo pipefail

root_dir="$(pwd -P)"
libname="libsolana_shmem_bridge"

case "$(uname -s)" in
  Darwin) ext="dylib" ;;
  Linux) ext="so" ;;
  *)
    echo "Unsupported OS: $(uname -s)" >&2
    exit 1
    ;;
esac

libpath="${root_dir}/target/release/${libname}.${ext}"

cat > "${root_dir}/config.json" <<EOF
{
  "libpath": "${libpath}"
}
EOF

echo "[shmem-bridge] Wrote config.json with libpath:"
echo "  ${libpath}"
