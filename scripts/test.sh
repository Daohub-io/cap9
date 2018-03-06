#! /bin/bash

output=$(nc -z localhost 8545; echo $?)
[ $output -eq "0" ] && trpc_running=true
if [ ! $trpc_running ]; then
  echo "Starting our own testrpc node instance"
  ./node_modules/.bin/ganache-cli > /dev/null &
  trpc_pid=$!
fi
./node_modules/.bin/truffle test
if [ ! $trpc_running ]; then
  kill -9 $trpc_pid
fi