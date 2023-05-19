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
echo '***' Starting crash test.

# generate the correct output
$SEQUENTIAL -a crash ../txts/pg*txt || exit 1
sort ./mr-crash-seq > mr-correct-crash.txt

rm -f mr-done
($COORDINATOR ../txts/pg*txt ; touch mr-done ) &
sleep 1

# start multiple workers
CRASH=1 $TIMEOUT2 $WORKER -a crash &

( while [ ! -f mr-done ]
  do
    CRASH=1 $TIMEOUT2 $WORKER -a crash || true
    sleep 1
  done ) &

( while [ ! -f mr-done ]
  do
    CRASH=1 $TIMEOUT2 $WORKER -a crash || true
    sleep 1
  done ) &

while [ ! -f mr-done ]
do
    CRASH=1 $TIMEOUT2 $WORKER -a crash || true
  sleep 1
done

wait

sort ./mr-out* | rg . > mr-crash-all
if cmp mr-crash-all mr-correct-crash.txt
then
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
