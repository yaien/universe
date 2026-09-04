build: esbuild
    cargo build

test:
    cargo test

esbuild:
    npx esbuild src/web/dashboard/assets/dashboard.ts --bundle --outfile=data/dist/dashboard.min.js --minify --sourcemap  --format=esm

run: esbuild
    cargo run

watch:
    watchexec -r --bell -e rs,ts,css -w src just run
