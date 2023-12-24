#!/usr/bin/env bash -v
ORIGINALWD=$(pwd)
cd "$(dirname "$0")"
mkdir build/
openscad -o build/case-top-scr.stl -D gen_top=true -D gen_scr=true case.scad
openscad -o build/case-top.stl -D gen_top=true case.scad
openscad -o build/case-bot.stl -D gen_bot=true case.scad
cd $ORIGINALWD
