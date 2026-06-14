#!/usr/bin/env bash

pkill powerexecd

rm -v -r ~/dotfiles/configs/sway/scripts/powerexecd*

cp -v ~/Projects/powerexecd/target/release/powerexecd ~/dotfiles/configs/sway/scripts/
