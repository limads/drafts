cargo generate-lockfile --offline
cargo build --release --offline --verbose \
    --config "source.vendored-sources.directory=\"deps\"" \
    --config "source.crates-io.replace-with = \"vendored-sources\"" \
    --config "source.\"https://github.com/limads/filecase\".replace-with = \"vendored-sources\"" \
    --config "source.\"https://github.com/limads/filecase\".git = \"https://github.com/limads/filecase\"" \
    --config "source.\"https://github.com/reknih/lipsum\".replace-with = \"vendored-sources\"" \
    --config "source.\"https://github.com/reknih/lipsum\".git = \"https://github.com/reknih/lipsum\"" \
    --config "source.\"https://github.com/typst/biblatex\".replace-with = \"vendored-sources\"" \
    --config "source.\"https://github.com/typst/biblatex\".git = \"https://github.com/typst/biblatex\"" \
    --config "source.\"https://github.com/typst/comemo\".replace-with = \"vendored-sources\"" \
    --config "source.\"https://github.com/typst/comemo\".git = \"https://github.com/typst/comemo\"" \
    --config "source.\"https://github.com/typst/hayagriva\".replace-with = \"vendored-sources\"" \
    --config "source.\"https://github.com/typst/hayagriva\".git = \"https://github.com/typst/hayagriva\"" \
    --config "source.\"https://github.com/typst/pixglyph\".replace-with = \"vendored-sources\"" \
    --config "source.\"https://github.com/typst/pixglyph\".git = \"https://github.com/typst/pixglyph\"" \
    --config "source.\"https://github.com/typst/typst\".replace-with = \"vendored-sources\"" \
    --config "source.\"https://github.com/typst/typst\".git = \"https://github.com/typst/typst\"" \
    --config "source.\"https://github.com/typst/unicode-math-class\".replace-with = \"vendored-sources\"" \
    --config "source.\"https://github.com/typst/unicode-math-class\".git = \"https://github.com/typst/unicode-math-class\""
