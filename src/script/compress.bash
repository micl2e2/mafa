cat $1  | terser -m | sed "s/\"/'/g"
