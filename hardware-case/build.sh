#!/usr/bin/env bash
ORIGINALWD=$(pwd)
cd "$(dirname "$0")"
mkdir build/
openscad -o build/case-top-scr.stl -D gen_top=true -D gen_scr=true case.scad
openscad -o build/case-bot-scr.stl -D gen_scr=true -D gen_bot=true case.scad
openscad -o build/case-top.stl -D gen_top=true case.scad
openscad -o build/case-bot.stl -D gen_bot=true case.scad
openscad -o build/case-leg.stl -D gen_leg=true case.scad
cd $ORIGINALWD
