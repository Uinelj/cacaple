.PHONY: build test serve clean

build:
	cd crate && wasm-pack build --target web --out-dir ../web/pkg

test:
	cd crate && cargo test

serve: build
	cd web && python3 -m http.server 8080

clean:
	cd crate && cargo clean
	rm -rf web/pkg
