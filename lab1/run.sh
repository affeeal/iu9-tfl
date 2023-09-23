#!/bin/bash

OUTPUT_FILE="output.smt2"
TEMPLATE_FILE="template.smt2"
INPUT_FILE=$([$1 == ''] && echo "input_sat.txt" || echo $1)

rlc main.ref && ./main $OUTPUT_FILE $TEMPLATE_FILE $INPUT_FILE
z3 -smt2 $OUTPUT_FILE
