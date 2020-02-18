#!/bin/bash

version="stable"
features="stm32f042,rt"

filename="bloat_log_"$version"_"`date -Iminutes`".txt"
filenamenoopt="bloat_noopt_log_"$version"_"`date -Iminutes`".txt"

cargo +$version rustc -- -V >>$filename
cargo +$version rustc -- -V >>$filenamenoopt

for i in `find examples -name "*.rs"`; do
        name=$(echo $i | sed -e "s,examples/,,g" -e "s,\.rs,,g")
        echo "Processing example $name"

        echo >>$filename
        echo "Bloat for example $name" >>$filename
        cargo +$version bloat --release --example $name --features="$features" -n 60 >>$filename
        echo >>$filename
        echo "Section sizes: " >>$filename
        cargo +$version size --release --example $name --features="$features" >>$filename

        echo >>$filenamenoopt
        echo "Bloat for example $name" >>$filenamenoopt
        cargo +$version bloat --example $name --features="$features" -n 60 >>$filenamenoopt
        echo >>$filenamenoopt
        echo "Section size: " >>$filenamenoopt
        cargo +$version size --example $name --features="$features" >>$filenamenoopt
done

echo "Captured bloat for rustc version \"$version\" for all examples with features \"$features\" into $filename and $filenamenoopt"
