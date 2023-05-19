#!/usr/bin/env bash

#
# basic map-reduce test
#

# killall dead processes
killall coordinator
killall worker
killall sequential

# make sure software is freshly built.
just build

# run the test in a fresh sub-directory.
cd mr-tmp || exit 1

failed_any=0

SEQUENTIAL=../target/release/sequential
COORDINATOR=../target/release/coordinator
WORKER=../target/release/worker
TIMEOUT="timeout -k 2s 45s "
TIMEOUT2="timeout -k 2s 120s "
#########################################################
# first word-count

# generate the correct output
$SEQUENTIAL -a wc ../txts/pg*txt
echo '***' Starting wc test.

$TIMEOUT $COORDINATOR ../txts/pg*txt &
pid=$!

# give the coordinator time to create the sockets.
sleep 1

# start multiple workers.
$TIMEOUT $WORKER -a wc &
$TIMEOUT $WORKER -a wc &
$TIMEOUT $WORKER -a wc &

# wait for the coordinator to exit.
wait $pid

# since workers are required to exit when a job is completely finished,
# and not before, that means the job has finished.
sort mr-out* | rg . >mr-wc-all
if cmp mr-wc-all mr-wc-seq; then
	echo '---' wc test: PASS
else
	echo '---' wc output is not the same as mr-wc-seq
	echo '---' wc test: FAIL
	failed_any=1
fi

# wait for remaining workers and coordinator to exit.
wait

#########################################################
# now indexer

just clean_log
just clean_tmp

# generate the correct output
$SEQUENTIAL -a indexer ../txts/pg*txt || exit 1
sort mr-indexer-seq >mr-correct-indexer.txt

echo '***' Starting indexer test.

$TIMEOUT $COORDINATOR ../txts/pg*txt &
sleep 1

# start multiple workers
$TIMEOUT $WORKER -a indexer &
$TIMEOUT $WORKER -a indexer

sort mr-out* | rg . >mr-indexer-all
if cmp mr-indexer-all mr-correct-indexer.txt; then
	echo '---' indexer test: PASS
else
	echo '---' indexer output is not the same as mr-correct-indexer.txt
	echo '---' indexer test: FAIL
	failed_any=1
fi

wait
#########################################################
# echo '***' Starting map parallelism test.
#
# rm -f mr-*
# rm -rf ../log
# mkdir ../log
#
# $TIMEOUT $COORDINATOR ../pg*txt &
# sleep 1
#
# $TIMEOUT $WORKER -a mtiming &
# $TIMEOUT $WORKER -a mtiming
#
# NT=$(cat mr-out* | rg '^times-' | wc -l | sed 's/ //g')
# if [ "$NT" != "2" ]; then
# 	echo '---' saw "$NT" workers rather than 2
# 	echo '---' map parallelism test: FAIL
# 	failed_any=1
# fi
#
# if cat mr-out* | rg '^parallel.* 2' >/dev/null; then
# 	echo '---' map parallelism test: PASS
# else
# 	echo '---' map workers did not run in parallel
# 	echo '---' map parallelism test: FAIL
# 	failed_any=1
# fi
#
# wait

#########################################################
just clean_log
just clean_tmp

echo '***' Starting job count test.

$TIMEOUT $COORDINATOR ../txts/pg*txt &
sleep 1

$TIMEOUT $WORKER -a jobcount &
$TIMEOUT $WORKER -a jobcount
$TIMEOUT $WORKER -a jobcount &
$TIMEOUT $WORKER -a jobcount

NT=$(cat mr-out* | awk '{print $2}')
if [ "$NT" -eq "8" ]; then
	echo '---' job count test: PASS
else
	echo '---' map jobs ran incorrect number of times "($NT != 8)"
	echo '---' job count test: FAIL
	failed_any=1
fi

wait

#########################################################
# test whether any worker or coordinator exits before the
# task has completed (i.e., all output files have been finalized)
just clean_log
just clean_tmp

echo '***' Starting early exit test.

$TIMEOUT $COORDINATOR ../txts/pg*txt &

# give the coordinator time to create the sockets.
sleep 1

# start multiple workers.
$TIMEOUT $WORKER -a early_exit &
pid=$!
$TIMEOUT $WORKER -a early_exit &
$TIMEOUT $WORKER -a early_exit &

# wait for any of the coord or workers to exit.
# `jobs` ensures that any completed old processes from other tests
# are not waited upon.
# jobs &>/dev/null
# the -n causes wait to wait for just one child process,
# rather than waiting for all to finish.
wait $pid

# a process has exited. this means that the output should be finalized
# otherwise, either a worker or the coordinator exited early
sort mr-out* | rg . >mr-wc-all-initial

# wait for remaining workers and coordinator to exit.
wait

# compare initial and final outputs
sort mr-out* | rg . >mr-wc-all-final
if cmp mr-wc-all-final mr-wc-all-initial; then
	echo '---' early exit test: PASS
else
	echo '---' output changed after first worker exited
	echo '---' early exit test: FAIL
	failed_any=1
fi

#########################################################
# test whether any worker or coordinator exits before the
# task has completed (i.e., all output files have been finalized)
just clean_log
just clean_tmp

echo '***' Starting early exit test.

$TIMEOUT $COORDINATOR ../txts/pg*txt &

# give the coordinator time to create the sockets.
sleep 1

# start multiple workers.
$TIMEOUT $WORKER -a early_exit &
pid=$!
$TIMEOUT $WORKER -a early_exit &
$TIMEOUT $WORKER -a early_exit &

# wait for any of the coord or workers to exit.
# `jobs` ensures that any completed old processes from other tests
# are not waited upon.
#jobs &>/dev/null
wait $pid

# a process has exited. this means that the output should be finalized
# otherwise, either a worker or the coordinator exited early
sort mr-out* | rg . >mr-wc-all-initial

# wait for remaining workers and coordinator to exit.
wait

# compare initial and final outputs
sort mr-out* | rg . >mr-wc-all-final
if cmp mr-wc-all-final mr-wc-all-initial; then
	echo '---' early exit test: PASS
else
	echo '---' output changed after first worker exited
	echo '---' early exit test: FAIL
	failed_any=1
fi
#########################################################
echo '***' Starting crash test.

# generate the correct output
$SEQUENTIAL -a crash ../txts/pg*txt || exit 1
sort ./mr-crash-seq >mr-correct-crash.txt

rm -f mr-done
(
	$COORDINATOR ../txts/pg*txt
	touch mr-done
) &
sleep 1

# start multiple workers
CRASH=1 $TIMEOUT2 $WORKER -a crash &

(while [ ! -f mr-done ]; do
	CRASH=1 $TIMEOUT2 $WORKER -a crash || true
	sleep 1
done) &

(while [ ! -f mr-done ]; do
	CRASH=1 $TIMEOUT2 $WORKER -a crash || true
	sleep 1
done) &

while [ ! -f mr-done ]; do
	CRASH=1 $TIMEOUT2 $WORKER -a crash || true
	sleep 1
done

wait

sort ./mr-out* | rg . >mr-crash-all
if cmp mr-crash-all mr-correct-crash.txt; then
	echo '---' crash test: PASS
else
	echo '---' crash output is not the same as mr-correct-crash.txt
	echo '---' crash test: FAIL
	failed_any=1
fi

#########################################################
if [ $failed_any -eq 0 ]; then
	echo '***' PASSED ALL TESTS
else
	echo '***' FAILED SOME TESTS
	exit 1
fi
