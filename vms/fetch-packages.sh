#!/bin/sh
# Downloads the .deb, the .rpm and their SHA256SUMS that CI built for a commit, and checks the
# sums (T-129). These are the files a release's smoke test and checklist run on, and that are
# then attached to its GitHub release (TestSetup.md, "A release, step by step").
#
#   vms/fetch-packages.sh              main's newest commit on GitHub
#   vms/fetch-packages.sh <commit>     that commit, such as a pull request's
#
# They go into target/github-packages/<commit>/, and the folder is printed last, for
# vms/smoke.py --packages. CI keeps them for 90 days for main, and 7 for a pull request.
set -eu
repo=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo"

if [ $# -gt 1 ]; then
    echo "Usage: $0 [commit]" >&2
    exit 2
fi
git fetch --quiet origin
commit=$(git rev-parse --verify "${1:-origin/main}^{commit}")

run=$(gh run list --workflow CI --commit "$commit" --status success --limit 1 \
    --json databaseId -q '.[0].databaseId')
if [ -z "$run" ]; then
    echo "No successful CI run for $commit yet; see: gh run list --commit $commit" >&2
    exit 1
fi

out="$repo/target/github-packages/$commit"
rm -rf "$out"
mkdir -p "$out"
echo "==> Downloading the packages of CI run $run, for $commit" >&2
if ! gh run download "$run" --name packages --dir "$out"; then
    rm -rf "$out"
    echo "CI run $run has no packages: it ran before T-129, or they have expired" >&2
    exit 1
fi
echo "==> Checking their SHA-256 sums" >&2
(cd "$out" && sha256sum --check --strict SHA256SUMS) >&2
echo "$out"
