#!/bin/bash
# generate some files and sleep

date

for i in $(seq 1 5); do
  echo $i
  echo $i >$i.txt
  sleep 1 
done

date
