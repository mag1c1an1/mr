APP = wc

build:
	cargo build --release

seq: build
	cargo run --release --package sequential -- -a ${APP} txts/*

dist: build clean
	make dist-coordinator &
	sleep 1
	make dist-workers
	sleep 1
	make merge

dist-coordinator:
	cargo run --release --package distributed --bin coordinator --  txts/* 

dist-worker:
	mkdir -p out
	cargo run --release --package distributed --bin worker -- -a ${APP}

dist-workers:
	make dist-worker &
	make dist-worker &
	make dist-worker

clean:
	rm -f mr-worker-*
	rm -f mr-out-0
	rm -f out/mr-*

merge:
	cd out && sort mr-out* | grep . > mr-all

diff:
	diff mr-out-0 out/mr-all