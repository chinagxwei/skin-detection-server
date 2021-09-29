#!/bin/bash

rw=`dirname $0`
cd $rw

nohup skin-detection-server & echo $! > pidfile.pid
