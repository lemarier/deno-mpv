#!/bin/sh
function replace_dlybs() {
	DYLIBS=`otool -L $1 | grep "/usr/local/Cellar" | awk -F' ' '{print \$1 }'`
	for dylib in $DYLIBS; do sudo install_name_tool -change $dylib @loader_path/`basename $dylib` $1; done;
	DYLIBS=`otool -L $1 | grep "/usr/local/opt" | awk -F' ' '{print \$1 }'`
	for dylib in $DYLIBS; do sudo install_name_tool -change $dylib @loader_path/`basename $dylib` $1; done;
}

replace_dlybs "target/release/libdeno_mpv.dylib"