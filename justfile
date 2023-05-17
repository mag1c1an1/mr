app: 
    @for file in `ls ./apps`; do \
    rustc --edition 2021 --out-dir ./applibs ./apps/$file; \
    done

build: clean app
    cargo build --release

seq: build
    cargo run --release --package sequential -- -a wc txts/*

clean_tmp:
    @rm -rf mr-tmp
    @mkdir mr-tmp

clean_app:
    @rm -rf applibs
    @mkdir applibs

clean_out:
    @rm -rf out
    @mkdir out

clean: clean_app clean_tmp clean_out
