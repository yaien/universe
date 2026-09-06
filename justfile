build: build-assets
    cargo build

test:
    cargo test

build-assets:
    npx esbuild src/web/dashboard/assets/dashboard.ts --bundle --outfile=data/dist/dashboard.min.js --minify --sourcemap  --format=esm

run: build-assets
    cargo run

watch:
    watchexec -r --bell -e rs,ts,css -w src just run
