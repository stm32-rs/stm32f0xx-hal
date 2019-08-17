#!/bin/bash

filename="bloat_log_"`date -Iminutes`".txt"
filenamenoopt="bloat_noopt_log_"`date -Iminutes`".txt"

for i in `find examples -name "*.rs"`; do
        name=$(echo $i | sed -e "s,examples/,,g" -e "s,\.rs,,g")
        echo "Processing example $name"
        echo >>$filename
        echo >>$filenamenoopt
        echo "Bloat for example $name" >>$filename
        echo "Bloat for example $name" >>$filenamenoopt
        cargo bloat --release --example $name --features="stm32f042,rt" -n 60 >>$filename
        cargo bloat --example $name --features="stm32f042,rt" -n 60 >>$filenamenoopt
done

echo "Captures bloat for all examples into $filename"
