#!/bin/bash

rw=`dirname $0`
cd $rw

kill -9 `cat pidfile.pid`
