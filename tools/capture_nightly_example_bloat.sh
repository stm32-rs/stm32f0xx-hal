#!/bin/bash

version="nightly"

filename="bloat_log_"$version"_"`date -Iminutes`".txt"
filenamenoopt="bloat_noopt_log_"$version"_"`date -Iminutes`".txt"

cargo +$version rustc -- -V >>$filename
cargo +$version rustc -- -V >>$filenamenoopt

for i in `find examples -name "*.rs"`; do
        name=$(echo $i | sed -e "s,examples/,,g" -e "s,\.rs,,g")
        echo "Processing example $name"
        echo >>$filename
        echo >>$filenamenoopt
        echo "Bloat for example $name" >>$filename
        echo "Bloat for example $name" >>$filenamenoopt
        cargo +$version bloat --release --example $name --features="stm32f042,rt" -n 60 >>$filename
        cargo +$version bloat --example $name --features="stm32f042,rt" -n 60 >>$filenamenoopt
done

echo "Captured bloat for rustc version \"$version\" for all examples into $filename and $filenamenoopt"
