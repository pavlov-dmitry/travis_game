#!/bin/sh
set -ex
curl -L http://downloads.sourceforge.net/project/libexif/libexif/0.6.21/libexif-0.6.21.zip > libexif.zip
unzip libexif.zip -d libexif
cd libexif/libexif-0.6.21 && ./configure --prefix=/usr --enable-shared && make && sudo make install
