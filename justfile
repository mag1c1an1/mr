APP := "wc"

build: clean 
    cargo build --release

seq: 
    cargo run --release --package sequential -- -a {{APP}}  txts/*

clean_tmp_dir:
    @rm -rf mr-tmp
    @mkdir mr-tmp

clean_tmp:
    @rm -f mr-tmp/mr-*

clean_log:
    @rm -rf log
    @mkdir log



clean: clean_tmp_dir clean_log
