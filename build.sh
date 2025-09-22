# !/usr/bin

make -C ./src
make clean -C ./src
rm ./fog
mv ./src/fog ./fog