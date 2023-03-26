mkdir -p build
flatpak-builder --repo=build/repo build/build io.github.limads.Drafts.json --state-dir=build/state --force-clean --install --user
