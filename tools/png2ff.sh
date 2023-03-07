#!/usr/bin/env bash

for f in images/*.png;
do
    filename=`basename -s .png ${f}`
    png2ff < ${f} > images/${filename}.ff;
done
