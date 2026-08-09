build: esbuild
    cargo build

test:
    cargo test

esbuild:
    npx esbuild src/web/dashboard/assets/dashboard.ts --bundle --outfile=src/web/dashboard/assets/dist/dashboard.min.js --minify --sourcemap  --format=esm

run: esbuild
    cargo run

watch:
    watchexec -r -e rs,ts,css -i "src/web/dashboard/assets/dist/*" just run
