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

#########################################################
# first word-count

# generate the correct output
just APP=wc seq || exit 1

echo '***' Starting wc test.

timeout -k 2s 180s just dist-coordinator &
pid=$!

# give the coordinator time to create the sockets.
sleep 1

# start multiple workers.
timeout -k 2s 180s just APP=wc dist-worker &
timeout -k 2s 180s just APP=wc dist-worker &
timeout -k 2s 180s just APP=wc dist-worker &

# wait for the coordinator to exit.
wait $pid

# since workers are required to exit when a job is completely finished,
# and not before, that means the job has finished.
sort ../out/mr-out* | grep . > mr-wc-all
if cmp mr-wc-all mr-correct-wc.txt
then
  echo '---' wc test: PASS
else
  echo '---' wc output is not the same as mr-correct-wc.txt
  echo '---' wc test: FAIL
  failed_any=1
fi

# wait for remaining workers and coordinator to exit.
wait