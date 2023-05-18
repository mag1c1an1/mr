APP := "wc"

app: 
    @for file in `ls ./apps`; do \
    rustc --edition 2021 --out-dir ./applibs ./apps/$file; \
    done

build: clean app
    cargo build --release

seq: 
    cargo run --release --package sequential -- -a {{APP}}  txts/*

dist: 
    just dist-coordinator
    sleep 1
    just dist-workers
    sleep 1
    just merge

dist-coordinator:
    cargo run --release --package distributed --bin coordinator -- txts/*

dist-worker:
    cargo run --release --package distributed --bin worker -- -a {{APP}} 

dist-workers:
    just dist-worker &    
    just dist-worker &    
    just dist-worker

merge:  
    cd out && sort mr-out* | rg . > mr-all

diff:
    diff out/mr-ans out/mr-all

clean_tmp:
    @rm -rf mr-tmp
    @mkdir mr-tmp

clean_app:
    @rm -rf applibs
    @mkdir applibs

clean_out:
    @rm -rf out
    @mkdir out

clean_log:
    @rm -rf log
    @mkdir log

clean: clean_app clean_tmp clean_out clean_log
