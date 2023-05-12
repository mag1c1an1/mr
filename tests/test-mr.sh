TIMEOUT=timeout
if timeout 2s sleep 1 >/dev/null 2>&1; then
	:
else
	if gtimeout 2s sleep 1 >/dev/null 2>&1; then
		TIMEOUT=gtimeout
	else
		# no timeout command
		TIMEOUT=
		echo '*** Cannot find timeout command; proceeding without timeouts.'
	fi
fi
if [ "$TIMEOUT" != "" ]; then
	TIMEOUT+=" -k 2s 180s "
fi

# fresh build
cargo build --release

# run the test in a fresh sub-directory
rm -rf mr-tmp
mkdir mr-tmp || exit 1
cd mr-tmp || exit 1
rm -f mr-*

failed_any=0

SEQUENTIAL=../target/release/sequential
COORDINATOR=../target/release/coordinator
WORKER=../target/release/worker
#########################################################
# first word-count

# generate the correct output
$SEQUENTIAL wc ../files/* || exit 1
sort mr-out-0 >mr-correct-wc.txt
rm -f mr-out*

echo '***' Starting wc test.

$TIMEOUT $COORDINATOR ../files/* &
pid=$!

# give the coordinator time to create the sockets.
sleep 1

# start multiple workers.
$TIMEOUT $WORKER wc &
$TIMEOUT $WORKER wc &
$TIMEOUT $WORKER wc &

# wait for the coordinator to exit.
wait $pid

# since workers are required to exit when a job is completely finished,
# and not before, that means the job has finished.
sort mr-out* | grep . >mr-wc-all
if cmp mr-wc-all mr-correct-wc.txt; then
	echo '---' wc test: PASS
else
	echo '---' wc output is not the same as mr-correct-wc.txt
	echo '---' wc test: FAIL
	failed_any=1
fi

# wait for remaining workers and coordinator to exit.
wait
